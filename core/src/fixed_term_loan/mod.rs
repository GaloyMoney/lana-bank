mod entity;
pub mod error;
mod job;
mod repo;
mod terms;

use job::*;
use sqlx::PgPool;
use tracing::instrument;

use crate::{
    entity::EntityError,
    job::{JobRegistry, Jobs},
    ledger::{fixed_term_loan::FixedTermLoanAccountIds, Ledger},
    primitives::*,
    user::UserRepo,
};

pub use entity::*;
use error::*;
use repo::*;
pub use terms::*;

#[derive(Clone)]
pub struct FixedTermLoans {
    repo: FixedTermLoanRepo,
    users: UserRepo,
    ledger: Ledger,
    jobs: Option<Jobs>,
    pool: PgPool,
}

impl FixedTermLoans {
    pub fn new(
        pool: &PgPool,
        registry: &mut JobRegistry,
        users: &UserRepo,
        ledger: &Ledger,
    ) -> Self {
        let repo = FixedTermLoanRepo::new(pool);
        registry.add_initializer(FixedTermLoanInterestJobInitializer::new(
            ledger,
            repo.clone(),
        ));
        Self {
            repo,
            users: users.clone(),
            ledger: ledger.clone(),
            jobs: None,
            pool: pool.clone(),
        }
    }

    pub fn set_jobs(&mut self, jobs: &Jobs) {
        self.jobs = Some(jobs.clone());
    }

    pub fn jobs(&self) -> &Jobs {
        self.jobs.as_ref().expect("Jobs must already be set")
    }

    #[instrument(name = "lava.fixed_term_loans.create_loan_for_user", skip(self), err)]
    pub async fn create_loan_for_user(
        &self,
        user_id: impl Into<UserId> + std::fmt::Debug,
    ) -> Result<FixedTermLoan, FixedTermLoanError> {
        let mut tx = self.pool.begin().await?;
        let loan_id = FixedTermLoanId::new();
        let new_loan = NewFixedTermLoan::builder()
            .id(loan_id)
            .user_id(user_id)
            .account_ids(FixedTermLoanAccountIds::new())
            .interest_interval(InterestInterval::Secondly)
            .rate(FixedTermLoanRate::from_bips(5))
            .build()
            .expect("Could not build FixedTermLoan");
        let loan = self.repo.create_in_tx(&mut tx, new_loan).await?;
        tx.commit().await?;
        Ok(loan)
    }

    #[instrument(name = "lava.fixed_term_loans.approve_loan", skip(self), err)]
    pub async fn approve_loan(
        &self,
        loan_id: impl Into<FixedTermLoanId> + std::fmt::Debug,
        collateral: Satoshis,
        principal: UsdCents,
    ) -> Result<FixedTermLoan, FixedTermLoanError> {
        let mut loan = self.repo.find_by_id(loan_id.into()).await?;
        let user = self.users.find_by_id(loan.user_id).await?;
        let mut tx = self.pool.begin().await?;
        let tx_id = LedgerTxId::new();
        loan.approve(tx_id, collateral, principal)?;
        self.repo.persist_in_tx(&mut tx, &mut loan).await?;
        self.ledger
            .create_accounts_for_fixed_term_loan(loan.id, loan.account_ids)
            .await?;
        self.ledger
            .approve_fixed_term_loan(
                tx_id,
                loan.account_ids,
                user.account_ids,
                collateral,
                principal,
                format!("{}-approval", loan.id),
            )
            .await?;
        self.jobs()
            .create_and_spawn_job_at::<FixedTermLoanInterestJobInitializer, _>(
                &mut tx,
                loan.id,
                format!("loan-interest-{}", loan.id),
                FixedTermLoanJobConfig { loan_id: loan.id },
                loan.next_interest_at().expect("first interest payment"),
            )
            .await?;
        tx.commit().await?;
        Ok(loan)
    }

    #[instrument(name = "lava.fixed_term_loans.record_payment", skip(self), err)]
    pub async fn record_payment(
        &self,
        loan_id: impl Into<FixedTermLoanId> + std::fmt::Debug,
        amount: UsdCents,
    ) -> Result<FixedTermLoan, FixedTermLoanError> {
        let mut loan = self.repo.find_by_id(loan_id.into()).await?;

        let balances = self
            .ledger
            .get_fixed_term_loan_balance(loan.account_ids)
            .await?;

        let tx_id = LedgerTxId::new();
        let tx_ref =
            loan.record_if_not_exceeding_outstanding(tx_id, balances.outstanding, amount)?;

        let user = self.users.find_by_id(loan.user_id).await?;

        let mut db_tx = self.pool.begin().await?;
        self.repo.persist_in_tx(&mut db_tx, &mut loan).await?;

        if !loan.is_completed() {
            self.ledger
                .record_payment_for_fixed_term_loan(
                    tx_id,
                    loan.account_ids,
                    user.account_ids,
                    amount,
                    tx_ref,
                )
                .await?;
        } else {
            self.ledger
                .complete_fixed_term_loan(
                    tx_id,
                    loan.account_ids,
                    user.account_ids,
                    amount,
                    balances.collateral,
                    tx_ref,
                )
                .await?;
        }
        db_tx.commit().await?;

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

    pub async fn list_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<FixedTermLoan>, FixedTermLoanError> {
        self.repo.list_for_user(user_id).await
    }
}
