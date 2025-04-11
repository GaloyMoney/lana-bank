mod config;
mod error;

use sqlx::PgPool;
use tracing::instrument;

use authz::PermissionCheck;

use crate::{
    accounting::Accounting,
    accounting_init::{ChartsInit, JournalInit, StatementsInit},
    applicant::Applicants,
    audit::{Audit, AuditCursor, AuditEntry},
    authorization::{init as init_authz, AppAction, AppObject, AuditAction, Authorization},
    balance_sheet::BalanceSheets,
    cash_flow::CashFlowStatements,
    credit_facility::CreditFacilities,
    customer::Customers,
    customer_onboarding::CustomerOnboarding,
    dashboard::Dashboard,
    deposit::Deposits,
    document::Documents,
    general_ledger::GeneralLedger,
    governance::Governance,
    job::Jobs,
    outbox::Outbox,
    price::Price,
    primitives::Subject,
    report::Reports,
    storage::Storage,
    terms_template::TermsTemplates,
    trial_balance::TrialBalances,
    user::Users,
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
    users: Users,
    credit_facilities: CreditFacilities,
    trial_balances: TrialBalances,
    balance_sheets: BalanceSheets,
    cash_flow_statements: CashFlowStatements,
    general_ledger: GeneralLedger,
    price: Price,
    report: Reports,
    terms_templates: TermsTemplates,
    documents: Documents,
    outbox: Outbox,
    governance: Governance,
    dashboard: Dashboard,
    _user_onboarding: UserOnboarding,
    _customer_onboarding: CustomerOnboarding,
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
        let price = Price::new();
        let storage = Storage::new(&config.storage);
        let documents = Documents::new(&pool, &storage, &authz);
        let report = Reports::init(&pool, &config.report, &authz, &jobs, &storage).await?;
        let users = Users::init(&pool, &authz, &outbox, config.user.superuser_email).await?;
        let user_onboarding =
            UserOnboarding::init(&jobs, &outbox, &users, config.user_onboarding).await?;

        let cala_config = cala_ledger::CalaLedgerConfig::builder()
            .pool(pool.clone())
            .exec_migrations(false)
            .build()
            .expect("cala config");
        let cala = cala_ledger::CalaLedger::init(cala_config).await?;
        let journal_init = JournalInit::journal(&cala).await?;
        let trial_balances =
            TrialBalances::init(&pool, &authz, &cala, journal_init.journal_id).await?;
        let balance_sheets =
            BalanceSheets::init(&pool, &authz, &cala, journal_init.journal_id).await?;
        let cash_flow_statements =
            CashFlowStatements::init(&pool, &authz, &cala, journal_init.journal_id).await?;
        let accounting = Accounting::new(&pool, &authz, &cala, journal_init.journal_id);
        StatementsInit::statements(
            &trial_balances,
            accounting.profit_and_loss(),
            &balance_sheets,
            &cash_flow_statements,
        )
        .await?;
        ChartsInit::charts_of_accounts(accounting.chart_of_accounts()).await?;
        let general_ledger = GeneralLedger::init(&authz, &cala, journal_init.journal_id);
        let customers = Customers::new(&pool, &authz, &outbox);
        let deposits = Deposits::init(
            &pool,
            &authz,
            &outbox,
            &governance,
            &jobs,
            &cala,
            journal_init.journal_id,
        )
        .await?;
        let customer_onboarding = CustomerOnboarding::init(
            &jobs,
            &outbox,
            &customers,
            &deposits,
            config.customer_onboarding,
        )
        .await?;
        let applicants =
            Applicants::init(&pool, &config.sumsub, &customers, &deposits, &jobs, &outbox).await?;

        let credit_facilities = CreditFacilities::init(
            &pool,
            config.credit_facility,
            &governance,
            &jobs,
            &authz,
            &customers,
            &price,
            &outbox,
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
            accounting,
            customers,
            deposits,
            applicants,
            users,
            price,
            report,
            credit_facilities,
            trial_balances,
            balance_sheets,
            cash_flow_statements,
            general_ledger,
            terms_templates,
            documents,
            outbox,
            governance,
            dashboard,
            _user_onboarding: user_onboarding,
            _customer_onboarding: customer_onboarding,
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
            .enforce_permission(sub, AppObject::Audit, AppAction::Audit(AuditAction::List))
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

    pub fn credit_facilities(&self) -> &CreditFacilities {
        &self.credit_facilities
    }

    pub fn trial_balances(&self) -> &TrialBalances {
        &self.trial_balances
    }

    pub fn balance_sheets(&self) -> &BalanceSheets {
        &self.balance_sheets
    }

    pub fn cash_flow_statements(&self) -> &CashFlowStatements {
        &self.cash_flow_statements
    }

    pub fn general_ledger(&self) -> &GeneralLedger {
        &self.general_ledger
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
