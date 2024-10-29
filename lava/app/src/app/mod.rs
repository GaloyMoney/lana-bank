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
    data_export::Export,
    deposit::Deposits,
    document::Documents,
    governance::Governance,
    job::Jobs,
    ledger::Ledger,
    loan::Loans,
    outbox::Outbox,
    price::Price,
    primitives::Subject,
    report::Reports,
    storage::Storage,
    terms_template::TermsTemplates,
    user::Users,
    withdraw::Withdraws,
};

pub use config::*;
use error::ApplicationError;

#[derive(Clone)]
pub struct LavaApp {
    _pool: PgPool,
    _jobs: Jobs,
    audit: Audit,
    authz: Authorization,
    loans: Loans,
    customers: Customers,
    withdraws: Withdraws,
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
}

impl LavaApp {
    pub async fn run(pool: PgPool, config: AppConfig) -> Result<Self, ApplicationError> {
        sqlx::migrate!().run(&pool).await?;

        let mut jobs = Jobs::new(&pool, config.job_execution);
        let export = Export::new(config.ledger.cala_url.clone(), &jobs);
        let audit = Audit::new(&pool);
        let authz = init_authz(&pool, &audit).await?;
        let outbox = Outbox::init(&pool).await?;
        let governance = Governance::new(&pool, &authz, &outbox);
        let ledger = Ledger::init(config.ledger, &authz).await?;
        let customers = Customers::new(&pool, &config.customer, &ledger, &authz, &audit, &export);
        let applicants = Applicants::new(&pool, &config.sumsub, &customers, &jobs, &export);
        let withdraws = Withdraws::init(
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
        let price = Price::init(&pool, &jobs, &export).await?;
        let storage = Storage::new(&config.storage);
        let documents = Documents::new(&pool, &storage, &authz);
        let report = Reports::init(&pool, &config.report, &authz, &audit, &jobs, &storage).await?;
        let users = Users::init(&pool, &authz, &outbox, config.user.superuser_email).await?;
        let credit_facilities = CreditFacilities::init(
            &pool,
            config.credit_facility,
            &governance,
            &jobs,
            &export,
            &authz,
            &audit,
            &customers,
            &ledger,
            &price,
            &outbox,
        )
        .await?;
        let terms_templates = TermsTemplates::new(&pool, &authz, &export);
        let loans = Loans::init(
            &pool,
            config.loan,
            &jobs,
            &customers,
            &ledger,
            &authz,
            &audit,
            &export,
            &price,
        )
        .await?;
        jobs.start_poll().await?;

        Ok(Self {
            _pool: pool,
            _jobs: jobs,
            audit,
            authz,
            customers,
            withdraws,
            deposits,
            loans,
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
        })
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

    #[instrument(name = "lava.audit.list_audit", skip(self), err)]
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

    pub fn withdraws(&self) -> &Withdraws {
        &self.withdraws
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

    pub fn loans(&self) -> &Loans {
        &self.loans
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
