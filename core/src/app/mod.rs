mod config;
mod error;

use sqlx::PgPool;
use tracing::instrument;

use crate::{
    applicant::Applicants,
    audit::{Audit, AuditCursor, AuditEntry},
    authorization::{Action, AuditAction, Authorization, Object},
    customer::Customers,
    data_export::Export,
    deposit::Deposits,
    job::Jobs,
    ledger::Ledger,
    loan::Loans,
    price::Price,
    primitives::Subject,
    report::Reports,
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
    price: Price,
    report: Reports,
}

impl LavaApp {
    pub async fn run(pool: PgPool, config: AppConfig) -> Result<Self, ApplicationError> {
        let mut jobs = Jobs::new(&pool, config.job_execution);
        let export = Export::new(config.ledger.cala_url.clone(), &jobs);
        let audit = Audit::new(&pool);
        let authz = Authorization::init(&pool, &audit).await?;
        let ledger = Ledger::init(config.ledger, &authz).await?;
        let customers = Customers::new(&pool, &config.customer, &ledger, &authz, &audit, &export);
        let applicants = Applicants::new(&pool, &config.sumsub, &customers);
        let withdraws = Withdraws::new(&pool, &customers, &ledger, &authz, &export);
        let deposits = Deposits::new(&pool, &customers, &ledger, &authz, &export);
        let price = Price::new();
        let report = Reports::new(&pool, &config.report, &authz, &jobs);
        let users = Users::init(&pool, config.user, &authz, &audit, &export).await?;
        let loans = Loans::new(
            &pool,
            config.loan,
            &jobs,
            &customers,
            &ledger,
            &authz,
            &audit,
            &export,
            &price,
            &users,
        );
        jobs.start_poll().await?;

        loans.spawn_global_jobs().await?;

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
        })
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
        query: crate::query::PaginatedQueryArgs<AuditCursor>,
    ) -> Result<crate::query::PaginatedQueryRet<AuditEntry, AuditCursor>, ApplicationError> {
        self.authz
            .check_permission(sub, Object::Audit, Action::Audit(AuditAction::List))
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

    pub fn users(&self) -> &Users {
        &self.users
    }

    pub fn authz(&self) -> &Authorization {
        &self.authz
    }
}
