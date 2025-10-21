mod config;
mod error;

use sqlx::PgPool;
use tracing::{Instrument, instrument};

use authz::PermissionCheck;

use rbac_types::{AuditAction, AuditEntityAction, AuditObject};

use crate::{
    access::Access,
    accounting::Accounting,
    accounting_init::{AccountingPeriodsInit, ChartsInit, JournalInit, StatementsInit},
    applicant::Applicants,
    audit::{Audit, AuditCursor, AuditEntry},
    authorization::{Authorization, seed},
    contract_creation::ContractCreation,
    credit::Credit,
    custody::Custody,
    customer::Customers,
    customer_sync::CustomerSync,
    dashboard::Dashboard,
    deposit::Deposits,
    deposit_sync::DepositSync,
    document::DocumentStorage,
    governance::Governance,
    job::Jobs,
    notification::Notification,
    outbox::Outbox,
    price::Price,
    primitives::Subject,
    public_id::PublicIds,
    report::Reports,
    storage::Storage,
    user_onboarding::UserOnboarding,
};

pub use config::*;
use error::ApplicationError;

#[derive(Clone)]
pub struct LanaApp {
    _pool: PgPool,
    jobs: Jobs,
    audit: Audit,
    authz: Authorization,
    accounting: Accounting,
    customers: Customers,
    deposits: Deposits,
    applicants: Applicants,
    access: Access,
    credit: Credit,
    custody: Custody,
    price: Price,
    outbox: Outbox,
    governance: Governance,
    dashboard: Dashboard,
    public_ids: PublicIds,
    contract_creation: ContractCreation,
    reports: Reports,
    _user_onboarding: UserOnboarding,
    _customer_sync: CustomerSync,
    _deposit_sync: DepositSync,
}

impl LanaApp {
    #[instrument(name = "lana_app.init", skip_all, err)]
    pub async fn init(pool: PgPool, config: AppConfig) -> Result<Self, ApplicationError> {
        sqlx::migrate!()
            .run(&pool)
            .instrument(tracing::info_span!("lana_app.migrations"))
            .await?;

        let audit = Audit::new(&pool);
        let outbox = Outbox::init(&pool).await?;
        let authz = Authorization::init(&pool, &audit).await?;

        let access = Access::init(
            &pool,
            config.access,
            rbac_types::LanaAction::action_descriptions(),
            seed::PREDEFINED_ROLES,
            &authz,
            &outbox,
        )
        .await?;

        let mut jobs = Jobs::init(
            job::JobSvcConfig::builder()
                .pool(pool.clone())
                .poller_config(config.job_poller)
                .build()
                .expect("Couldn't build JobSvcConfig"),
        )
        .await?;

        let dashboard = Dashboard::init(&pool, &authz, &jobs, &outbox).await?;
        let governance = Governance::new(&pool, &authz, &outbox);
        let storage = Storage::new(&config.storage);
        let reports = Reports::init(&pool, &authz, config.report, &outbox, &jobs, &storage).await?;
        let price = Price::new();
        let documents = DocumentStorage::new(&pool, &storage);
        let public_ids = PublicIds::new(&pool);

        let user_onboarding = UserOnboarding::init(&jobs, &outbox, config.user_onboarding).await?;

        let cala_config = cala_ledger::CalaLedgerConfig::builder()
            .pool(pool.clone())
            .exec_migrations(false)
            .build()
            .expect("cala config");
        let cala = cala_ledger::CalaLedger::init(cala_config).await?;
        let journal_init = JournalInit::journal(&cala).await?;
        let accounting = Accounting::new(
            &pool,
            &authz,
            &cala,
            journal_init.journal_id,
            documents.clone(),
            &jobs,
        );

        StatementsInit::statements(&accounting).await?;

        let customers = Customers::new(
            &pool,
            &authz,
            &outbox,
            documents.clone(),
            public_ids.clone(),
        );
        let deposits = Deposits::init(
            &pool,
            &authz,
            &outbox,
            &governance,
            &jobs,
            &cala,
            journal_init.journal_id,
            &public_ids,
        )
        .await?;
        let customer_sync =
            CustomerSync::init(&jobs, &outbox, &customers, &deposits, config.customer_sync).await?;

        let applicants = Applicants::new(&pool, &config.sumsub, &authz, &customers);

        let deposit_sync = DepositSync::init(
            &jobs,
            &outbox,
            &deposits,
            &customers,
            crate::applicant::SumsubClient::new(&config.sumsub),
        )
        .await?;

        let custody = Custody::init(&pool, &authz, config.custody, &outbox).await?;

        let credit = Credit::init(
            &pool,
            config.credit,
            &governance,
            &jobs,
            &authz,
            &customers,
            &custody,
            &price,
            &outbox,
            &cala,
            journal_init.journal_id,
            &public_ids,
        )
        .await?;

        let contract_creation =
            ContractCreation::new(&customers, &applicants, &documents, &jobs, &authz);

        Notification::init(
            config.notification,
            &jobs,
            &outbox,
            access.users(),
            &credit,
            &customers,
        )
        .await?;

        let chart =
            ChartsInit::charts_of_accounts(&accounting, &credit, &deposits, config.accounting_init)
                .await?;

        AccountingPeriodsInit::open_initial_accounting_periods(&accounting, &chart).await?;

        jobs.start_poll().await?;

        Ok(Self {
            _pool: pool,
            jobs,
            audit,
            authz,
            accounting,
            customers,
            deposits,
            applicants,
            access,
            price,
            credit,
            custody,
            outbox,
            governance,
            dashboard,
            public_ids,
            contract_creation,
            reports,
            _user_onboarding: user_onboarding,
            _customer_sync: customer_sync,
            _deposit_sync: deposit_sync,
        })
    }

    pub fn dashboard(&self) -> &Dashboard {
        &self.dashboard
    }

    pub fn governance(&self) -> &Governance {
        &self.governance
    }

    pub fn reports(&self) -> &Reports {
        &self.reports
    }

    pub fn customers(&self) -> &Customers {
        &self.customers
    }

    pub fn audit(&self) -> &Audit {
        &self.audit
    }

    pub fn price(&self) -> &Price {
        &self.price
    }

    pub fn outbox(&self) -> &Outbox {
        &self.outbox
    }

    #[instrument(name = "lana.audit.list_audit", skip(self), err)]
    pub async fn list_audit(
        &self,
        sub: &Subject,
        query: es_entity::PaginatedQueryArgs<AuditCursor>,
    ) -> Result<es_entity::PaginatedQueryRet<AuditEntry, AuditCursor>, ApplicationError> {
        use crate::audit::AuditSvc;

        self.authz
            .enforce_permission(
                sub,
                AuditObject::all_audits(),
                AuditAction::from(AuditEntityAction::List),
            )
            .await?;

        self.audit.list(query).await.map_err(ApplicationError::from)
    }

    pub fn accounting(&self) -> &Accounting {
        &self.accounting
    }

    pub fn deposits(&self) -> &Deposits {
        &self.deposits
    }

    pub fn applicants(&self) -> &Applicants {
        &self.applicants
    }

    pub fn custody(&self) -> &Custody {
        &self.custody
    }

    pub fn credit(&self) -> &Credit {
        &self.credit
    }

    pub fn access(&self) -> &Access {
        &self.access
    }

    pub fn public_ids(&self) -> &PublicIds {
        &self.public_ids
    }

    pub fn contract_creation(&self) -> &ContractCreation {
        &self.contract_creation
    }

    pub async fn get_visible_nav_items(
        &self,
        sub: &Subject,
    ) -> Result<
        crate::authorization::VisibleNavigationItems,
        crate::authorization::error::AuthorizationError,
    > {
        crate::authorization::get_visible_navigation_items(&self.authz, sub).await
    }

    pub async fn shutdown(&self) -> Result<(), ApplicationError> {
        tracing::info!("app.shutdown");

        self.jobs.shutdown().await?;
        Ok(())
    }
}
