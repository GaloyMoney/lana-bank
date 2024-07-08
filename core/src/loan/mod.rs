mod entity;
pub mod error;
mod job;
mod repo;
mod terms;

use sqlx::PgPool;

use crate::{
    job::{JobRegistry, Jobs},
    ledger::{loan::*, Ledger},
    primitives::*,
    user::Users,
};

pub use entity::*;
use error::*;
use job::*;
use repo::*;
pub use terms::*;

#[derive(Clone)]
pub struct Loans {
    loan_repo: LoanRepo,
    term_repo: TermRepo,
    users: Users,
    ledger: Ledger,
    pool: PgPool,
    jobs: Option<Jobs>,
}

impl Loans {
    pub fn new(pool: &PgPool, registry: &mut JobRegistry, users: &Users, ledger: &Ledger) -> Self {
        let loan_repo = LoanRepo::new(pool);
        let term_repo = TermRepo::new(pool);
        registry.add_initializer(LoanProcessingJobInitializer::new(ledger, loan_repo.clone()));
        Self {
            loan_repo,
            term_repo,
            users: users.clone(),
            ledger: ledger.clone(),
            pool: pool.clone(),
            jobs: None,
        }
    }

    pub fn set_jobs(&mut self, jobs: &Jobs) {
        self.jobs = Some(jobs.clone());
    }

    fn jobs(&self) -> &Jobs {
        self.jobs.as_ref().expect("Jobs must already be set")
    }

    pub async fn update_current_terms(&self, terms: TermValues) -> Result<Terms, LoanError> {
        self.term_repo.update_current(terms).await
    }

    fn dummy_price() -> PriceOfOneBTC {
        PriceOfOneBTC::new(UsdCents::from_usd(rust_decimal_macros::dec!(60000)))
    }

    pub async fn create_loan_for_user(
        &self,
        user_id: impl Into<UserId>,
        desired_principal: UsdCents,
    ) -> Result<Loan, LoanError> {
        let user_id = user_id.into();
        let user = match self.users.find_by_id(user_id).await? {
            Some(user) => user,
            None => return Err(LoanError::UserNotFound(user_id)),
        };

        if !user.may_create_loan() {
            return Err(LoanError::UserNotAllowedToCreateLoan(user_id));
        }
        let unallocated_collateral = self
            .ledger
            .get_user_balance(user.account_ids)
            .await?
            .btc_balance;

        let current_terms = self.term_repo.find_current().await?;
        let required_collateral =
            current_terms.required_collateral(desired_principal, Self::dummy_price());

        if required_collateral > unallocated_collateral {
            return Err(LoanError::InsufficientCollateral(
                required_collateral,
                unallocated_collateral,
            ));
        }

        let mut tx = self.pool.begin().await?;

        let new_loan = NewLoan::builder()
            .id(LoanId::new())
            .user_id(user_id)
            .terms(current_terms.values)
            .principal(desired_principal)
            .initial_collateral(required_collateral)
            .account_ids(LoanAccountIds::new())
            .user_account_ids(user.account_ids)
            .build()
            .expect("could not build new loan");
        let loan = self.loan_repo.create_in_tx(&mut tx, new_loan).await?;
        self.ledger
            .create_accounts_for_loan(loan.id, loan.account_ids)
            .await?;
        self.jobs()
            .create_and_spawn_job::<LoanProcessingJobInitializer, _>(
                &mut tx,
                loan.id,
                format!("loan-processing-{}", loan.id),
                LoanJobConfig { loan_id: loan.id },
            )
            .await?;
        tx.commit().await?;
        Ok(loan)
    }

    pub async fn record_payment(
        &self,
        loan_id: LoanId,
        amount: UsdCents,
    ) -> Result<Loan, LoanError> {
        let mut loan = self.loan_repo.find_by_id(loan_id).await?;
        let balances = self.ledger.get_loan_balance(loan.account_ids).await?;

        let tx_id = LedgerTxId::new();
        let tx_ref =
            loan.record_if_not_exceeding_outstanding(tx_id, balances.outstanding, amount)?;

        let user = self.users.repo().find_by_id(loan.user_id).await?;
        let mut db_tx = self.pool.begin().await?;
        self.loan_repo.persist_in_tx(&mut db_tx, &mut loan).await?;
        if !loan.is_completed() {
            self.ledger
                .record_payment(tx_id, loan.account_ids, user.account_ids, amount, tx_ref)
                .await?;
        } else {
            self.ledger
                .complete_loan(
                    tx_id,
                    loan.account_ids,
                    user.account_ids,
                    amount,
                    balances.collateral,
                    tx_ref,
                )
                .await?;
            db_tx.commit().await?;
        }

        Ok(loan)
    }
}
