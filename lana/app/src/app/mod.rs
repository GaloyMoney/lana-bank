mod config;
mod error;

use sqlx::PgPool;
use tracing::instrument;

use authz::PermissionCheck;

use crate::{
    accounting_init::{ChartsInit, JournalInit},
    applicant::Applicants,
    audit::{Audit, AuditCursor, AuditEntry},
    authorization::{init as init_authz, AppAction, AppObject, AuditAction, Authorization},
    chart_of_accounts::ChartOfAccounts,
    credit_facility::CreditFacilities,
    customer::Customers,
    dashboard::Dashboard,
    deposit::Deposits,
    document::Documents,
    governance::Governance,
    job::Jobs,
    outbox::Outbox,
    price::Price,
    primitives::Subject,
    report::Reports,
    storage::Storage,
    terms_template::TermsTemplates,
    user::Users,
};

pub use config::*;
use error::ApplicationError;

#[derive(Clone)]
pub struct LanaApp {
    _pool: PgPool,
    _jobs: Jobs,
    audit: Audit,
    authz: Authorization,
    chart_of_accounts: ChartOfAccounts,
    customers: Customers,
    deposits: Deposits,
    applicants: Applicants,
    users: Users,
    credit_facilities: CreditFacilities,
    price: Price,
    report: Reports,
    terms_templates: TermsTemplates,
    documents: Documents,
    _outbox: Outbox,
    governance: Governance,
    dashboard: Dashboard,
}

impl LanaApp {
    pub async fn run(pool: PgPool, config: AppConfig) -> Result<Self, ApplicationError> {
        sqlx::migrate!().run(&pool).await?;

        let mut jobs = Jobs::new(&pool, config.job_execution);
        let audit = Audit::new(&pool);
        let authz = init_authz(&pool, &audit).await?;
        let outbox = Outbox::init(&pool).await?;
        let dashboard = Dashboard::init(&pool, &authz, &jobs, &outbox).await?;
        let governance = Governance::new(&pool, &authz, &outbox);
        let price = Price::init(&jobs).await?;
        let storage = Storage::new(&config.storage);
        let documents = Documents::new(&pool, &storage, &authz);
        let report = Reports::init(&pool, &config.report, &authz, &jobs, &storage).await?;
        let users = Users::init(&pool, &authz, &outbox, config.user.superuser_email).await?;

        let cala_config = cala_ledger::CalaLedgerConfig::builder()
            .pool(pool.clone())
            .exec_migrations(false)
            .build()
            .expect("cala config");
        let cala = cala_ledger::CalaLedger::init(cala_config).await?;
        let journal_init = JournalInit::journal(&cala).await?;
        let chart_of_accounts =
            ChartOfAccounts::init(&pool, &authz, &cala, journal_init.journal_id).await?;
        let charts_init = ChartsInit::charts_of_accounts(&chart_of_accounts).await?;

        let deposits_factory =
            chart_of_accounts.transaction_account_factory(charts_init.deposits.deposits);
        let deposits = Deposits::init(
            &pool,
            &authz,
            &outbox,
            &governance,
            &jobs,
            deposits_factory,
            &cala,
            journal_init.journal_id,
            String::from("OMNIBUS_ACCOUNT_ID"),
        )
        .await?;
        let customers = Customers::new(&pool, &config.customer, &deposits, &authz);
        let applicants = Applicants::new(&pool, &config.sumsub, &customers, &jobs);

        let collateral_factory =
            chart_of_accounts.transaction_account_factory(charts_init.credit_facilities.collateral);
        let facility_factory =
            chart_of_accounts.transaction_account_factory(charts_init.credit_facilities.facility);
        let disbursed_receivable_factory = chart_of_accounts
            .transaction_account_factory(charts_init.credit_facilities.disbursed_receivable);
        let interest_receivable_factory = chart_of_accounts
            .transaction_account_factory(charts_init.credit_facilities.interest_receivable);
        let interest_income_factory = chart_of_accounts
            .transaction_account_factory(charts_init.credit_facilities.interest_income);
        let fee_income_factory =
            chart_of_accounts.transaction_account_factory(charts_init.credit_facilities.fee_income);
        let credit_facilities = CreditFacilities::init(
            &pool,
            config.credit_facility,
            &governance,
            &jobs,
            &authz,
            &deposits,
            &price,
            &outbox,
            collateral_factory,
            facility_factory,
            disbursed_receivable_factory,
            interest_receivable_factory,
            interest_income_factory,
            fee_income_factory,
            &cala,
            journal_init.journal_id,
        )
        .await?;
        let terms_templates = TermsTemplates::new(&pool, &authz);
        jobs.start_poll().await?;

        Ok(Self {
            _pool: pool,
            _jobs: jobs,
            audit,
            authz,
            chart_of_accounts,
            customers,
            deposits,
            applicants,
            users,
            price,
            report,
            credit_facilities,
            terms_templates,
            documents,
            _outbox: outbox,
            governance,
            dashboard,
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

    #[instrument(name = "lana.audit.list_audit", skip(self), err)]
    pub async fn list_audit(
        &self,
        sub: &Subject,
        query: es_entity::PaginatedQueryArgs<AuditCursor>,
    ) -> Result<es_entity::PaginatedQueryRet<AuditEntry, AuditCursor>, ApplicationError> {
        use crate::audit::AuditSvc;

        self.authz
            .enforce_permission(sub, AppObject::Audit, AppAction::Audit(AuditAction::List))
            .await?;

        self.audit.list(query).await.map_err(ApplicationError::from)
    }

    pub fn chart_of_accounts(&self) -> &ChartOfAccounts {
        &self.chart_of_accounts
    }

    pub fn deposits(&self) -> &Deposits {
        &self.deposits
    }

    pub fn applicants(&self) -> &Applicants {
        &self.applicants
    }

    pub fn credit_facilities(&self) -> &CreditFacilities {
        &self.credit_facilities
    }

    pub fn users(&self) -> &Users {
        &self.users
    }

    pub fn terms_templates(&self) -> &TermsTemplates {
        &self.terms_templates
    }

    pub fn documents(&self) -> &Documents {
        &self.documents
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
