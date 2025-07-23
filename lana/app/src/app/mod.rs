mod config;
mod error;

use sqlx::PgPool;
use tracing::instrument;

use authz::PermissionCheck;

use crate::{
    access::Access,
    accounting::Accounting,
    accounting_init::{ChartsInit, JournalInit, StatementsInit},
    applicant::Applicants,
    audit::{Audit, AuditCursor, AuditEntry},
    authorization::{AppAction, AppObject, AuditAction, Authorization, seed},
    contract_creation::ContractCreation,
    credit::Credit,
    custody::Custody,
    customer::Customers,
    customer_sync::CustomerSync,
    dashboard::Dashboard,
    deposit::Deposits,
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
    _jobs: Jobs,
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
    report: Reports,
    outbox: Outbox,
    governance: Governance,
    dashboard: Dashboard,
    public_ids: PublicIds,
    contract_creation: ContractCreation,
    _user_onboarding: UserOnboarding,
    _customer_sync: CustomerSync,
}

impl LanaApp {
    pub async fn run(pool: PgPool, config: AppConfig) -> Result<Self, ApplicationError> {
        sqlx::migrate!().run(&pool).await?;

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

        let mut jobs = Jobs::new(&pool, config.job_execution);

        let dashboard = Dashboard::init(&pool, &authz, &jobs, &outbox).await?;
        let governance = Governance::new(&pool, &authz, &outbox);
        let price = Price::new();
        let storage = Storage::new(&config.storage);
        let documents = DocumentStorage::new(&pool, &storage);
        let public_ids = PublicIds::new(&pool);
        let report = Reports::init(&pool, &config.report, &authz, &jobs, &storage).await?;

        let user_onboarding =
            UserOnboarding::init(&jobs, &outbox, access.users(), config.user_onboarding).await?;

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

        let applicants = Applicants::new(&pool, &config.sumsub);

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
            ContractCreation::try_new(&customers, &applicants, &documents, &jobs, &authz)?;

        Notification::init(
            config.notification,
            &jobs,
            &outbox,
            access.users(),
            &credit,
            &customers,
        )
        .await?;
        ChartsInit::charts_of_accounts(&accounting, &credit, &deposits, config.accounting_init)
            .await?;

        jobs.start_poll().await?;

        Ok(Self {
            _pool: pool,
            _jobs: jobs,
            audit,
            authz,
            accounting,
            customers,
            deposits,
            applicants,
            access,
            price,
            report,
            credit,
            custody,
            outbox,
            governance,
            dashboard,
            public_ids,
            contract_creation,
            _user_onboarding: user_onboarding,
            _customer_sync: customer_sync,
        })
    }

    pub fn dashboard(&self) -> &Dashboard {
        &self.dashboard
    }

    pub fn governance(&self) -> &Governance {
        &self.governance
    }

    pub fn customers(&self) -> &Customers {
        &self.customers
    }

    pub fn audit(&self) -> &Audit {
        &self.audit
    }

    pub fn reports(&self) -> &Reports {
        &self.report
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
                AppObject::all_audits(),
                AppAction::Audit(AuditAction::List),
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
}
