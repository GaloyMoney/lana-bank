mod config;
mod error;

use sqlx::PgPool;
use tracing::instrument;

use authz::PermissionCheck;

use crate::{
    applicant::Applicants,
    audit::{Audit, AuditCursor, AuditEntry},
    authorization::{init as init_authz, AppAction, AppObject, AuditAction, Authorization},
    credit_facility::CreditFacilities,
    customer::Customers,
    dashboard::Dashboard,
    data_export::Export,
    deposit::Deposits,
    document::Documents,
    governance::Governance,
    job::Jobs,
    ledger::Ledger,
    outbox::Outbox,
    price::Price,
    primitives::Subject,
    report::Reports,
    storage::Storage,
    terms_template::TermsTemplates,
    user::Users,
    withdrawal::Withdrawals,
};

pub use config::*;
use error::ApplicationError;

#[derive(Clone)]
pub struct LanaApp {
    _pool: PgPool,
    _jobs: Jobs,
    audit: Audit,
    authz: Authorization,
    customers: Customers,
    withdrawals: Withdrawals,
    deposits: Deposits,
    ledger: Ledger,
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
        let export = Export::new(config.ledger.cala_url.clone(), &jobs);
        let audit = Audit::new(&pool);
        let authz = init_authz(&pool, &audit).await?;
        let outbox = Outbox::init(&pool).await?;
        let dashboard = Dashboard::init(&pool, &authz, &jobs, &outbox).await?;
        let governance = Governance::new(&pool, &authz, &outbox);
        let ledger = Ledger::init(config.ledger, &authz).await?;
        let customers = Customers::new(&pool, &config.customer, &ledger, &authz, &export);
        let applicants = Applicants::new(&pool, &config.sumsub, &customers, &jobs, &export);
        let withdrawals = Withdrawals::init(
            &pool,
            &customers,
            &ledger,
            &authz,
            &export,
            &governance,
            &jobs,
            &outbox,
        )
        .await?;
        let deposits = Deposits::new(&pool, &customers, &ledger, &authz, &export);
        let price = Price::init(&jobs, &export).await?;
        let storage = Storage::new(&config.storage);
        let documents = Documents::new(&pool, &storage, &authz);
        let report = Reports::init(&pool, &config.report, &authz, &jobs, &storage, &export).await?;
        let users = Users::init(&pool, &authz, &outbox, config.user.superuser_email).await?;
        let credit_facilities = CreditFacilities::init(
            &pool,
            config.credit_facility,
            &governance,
            &jobs,
            &export,
            &authz,
            &customers,
            &ledger,
            &price,
            &outbox,
        )
        .await?;
        let terms_templates = TermsTemplates::new(&pool, &authz, &export);
        jobs.start_poll().await?;

        Ok(Self {
            _pool: pool,
            _jobs: jobs,
            audit,
            authz,
            customers,
            withdrawals,
            deposits,
            ledger,
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

    pub fn withdrawals(&self) -> &Withdrawals {
        &self.withdrawals
    }

    pub fn deposits(&self) -> &Deposits {
        &self.deposits
    }

    pub fn ledger(&self) -> &Ledger {
        &self.ledger
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
