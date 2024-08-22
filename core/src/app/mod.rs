mod config;
mod error;

use sqlx::PgPool;

use crate::{
    applicant::Applicants,
    audit::{Audit, AuditCursor, AuditEntry},
    authorization::{Action, AuditAction, Authorization, Object},
    customer::Customers,
    deposit::Deposits,
    job::{JobRegistry, Jobs},
    ledger::Ledger,
    loan::Loans,
    primitives::Subject,
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
}

impl LavaApp {
    pub async fn run(pool: PgPool, config: AppConfig) -> Result<Self, ApplicationError> {
        let audit = Audit::new(&pool);
        let authz = Authorization::init(&pool, &audit).await?;
        let mut registry = JobRegistry::new();
        let ledger = Ledger::init(config.ledger, &authz).await?;
        let customers = Customers::new(&pool, &ledger, &config.customer, &authz, &audit);
        let applicants = Applicants::new(&pool, &config.sumsub, &customers);
        let withdraws = Withdraws::new(&pool, &customers, &ledger, &authz);
        let deposits = Deposits::new(&pool, &customers, &ledger, &authz);
        let mut loans = Loans::new(&pool, &mut registry, &customers, &ledger, &authz, &audit);
        let mut jobs = Jobs::new(&pool, config.job_execution, registry);
        let users = Users::init(&pool, config.user, &authz, &audit).await?;
        loans.set_jobs(&jobs);
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
        })
    }

    pub fn customers(&self) -> &Customers {
        &self.customers
    }

    pub fn audit(&self) -> &Audit {
        &self.audit
    }

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
}
