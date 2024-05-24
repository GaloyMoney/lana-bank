mod entity;
pub mod error;
mod job;
mod repo;
mod state;

use sqlx::PgPool;
use tracing::instrument;

use crate::{
    entity::{EntityError, EntityUpdate},
    job::{JobRegistry, Jobs},
    ledger::Ledger,
    primitives::*,
};

pub use entity::*;
use error::*;
use job::*;
use repo::*;
pub use state::*;

#[derive(Clone)]
pub struct FixedTermLoans {
    repo: FixedTermLoanRepo,
    ledger: Ledger,
    jobs: Option<Jobs>,
    pool: PgPool,
}

impl FixedTermLoans {
    pub fn new(pool: &PgPool, registry: &mut JobRegistry, ledger: Ledger) -> Self {
        let repo = FixedTermLoanRepo::new(pool);
        registry.add_initializer(FixedTermLoanJobInitializer::new(&ledger, repo.clone()));
        Self {
            repo,
            ledger,
            jobs: None,
            pool: pool.clone(),
        }
    }

    pub fn set_jobs(&mut self, jobs: &Jobs) {
        self.jobs = Some(jobs.clone());
    }

    fn jobs(&self) -> &Jobs {
        self.jobs.as_ref().expect("Jobs not set")
    }

    #[instrument(name = "lava.fixed_term_loans.create_loan", skip(self), err)]
    pub async fn create_loan(&self) -> Result<FixedTermLoan, FixedTermLoanError> {
        let loan_id = FixedTermLoanId::new();
        let new_loan = NewFixedTermLoan::builder()
            .id(loan_id)
            .build()
            .expect("Could not build FixedTermLoan");
        let mut tx = self.pool.begin().await?;
        let EntityUpdate { entity: loan, .. } = self.repo.create_in_tx(&mut tx, new_loan).await?;
        self.jobs()
            .create_and_spawn_job::<FixedTermLoanJobInitializer, _>(
                &mut tx,
                loan.id,
                format!("fixed_term_loan:{}", loan.id),
                FixedTermLoanJobConfig { loan_id: loan.id },
            )
            .await?;
        tx.commit().await?;
        Ok(loan)
    }

    pub async fn declare_collateralized(
        &self,
        id: FixedTermLoanId,
    ) -> Result<FixedTermLoan, FixedTermLoanError> {
        let mut loan = self.repo.find_by_id(id).await?;
        loan.declare_collateralized()?;
        self.repo.persist(&mut loan).await?;
        Ok(loan)
    }

    pub async fn find_by_id(
        &self,
        id: FixedTermLoanId,
    ) -> Result<Option<FixedTermLoan>, FixedTermLoanError> {
        match self.repo.find_by_id(id).await {
            Ok(loan) => Ok(Some(loan)),
            Err(FixedTermLoanError::EntityError(EntityError::NoEntityEventsPresent)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn balance_for_loan(
        &self,
        loan_id: FixedTermLoanId,
    ) -> Result<Money, FixedTermLoanError> {
        let loan = self.repo.find_by_id(loan_id).await?;
        let balance = self.ledger.fetch_btc_account_balance(loan.id).await?;
        Ok(balance)
    }
}
