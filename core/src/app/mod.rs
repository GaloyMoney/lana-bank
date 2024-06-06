mod config;
mod error;

use sqlx::PgPool;

use crate::{
    bank::Banks,
    fixed_term_loan::{FixedTermLoan, FixedTermLoans},
    job::{JobRegistry, Jobs},
    ledger::Ledger,
    primitives::UserId,
    user::Users,
    withdraw::Withdraws,
};

pub use config::*;
use error::ApplicationError;

#[derive(Clone)]
pub struct LavaApp {
    _pool: PgPool,
    _jobs: Jobs,
    banks: Banks,
    fixed_term_loans: FixedTermLoans,
    users: Users,
    withdraws: Withdraws,
    ledger: Ledger,
}

impl LavaApp {
    pub async fn run(pool: PgPool, config: AppConfig) -> Result<Self, ApplicationError> {
        let mut registry = JobRegistry::new();
        let ledger = Ledger::init(config.ledger).await?;
        let banks = Banks::new(&pool, &ledger);
        let users = Users::new(&pool, &ledger);
        let withdraws = Withdraws::new(&pool, users.repo(), &ledger);
        let mut fixed_term_loans = FixedTermLoans::new(&pool, &mut registry, users.repo(), &ledger);
        let mut jobs = Jobs::new(&pool, config.job_execution, registry);
        fixed_term_loans.set_jobs(&jobs);
        jobs.start_poll().await?;
        Ok(Self {
            _pool: pool,
            _jobs: jobs,
            banks,
            users,
            withdraws,
            fixed_term_loans,
            ledger,
        })
    }

    pub fn fixed_term_loans(&self) -> &FixedTermLoans {
        &self.fixed_term_loans
    }

    pub fn users(&self) -> &Users {
        &self.users
    }

    pub fn banks(&self) -> &Banks {
        &self.banks
    }

    pub fn withdraws(&self) -> &Withdraws {
        &self.withdraws
    }

    pub fn ledger(&self) -> &Ledger {
        &self.ledger
    }

    pub async fn list_loans_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Option<Vec<FixedTermLoan>>, ApplicationError> {
        if self.users().find_by_id(user_id).await?.is_some() {
            return Ok(Some(self.fixed_term_loans().list_for_user(user_id).await?));
        }
        Ok(None)
    }
}
