mod entity;
pub mod error;
mod job;
mod repo;
mod terms;

mod cursor;
pub use cursor::LoanCursor;

use sqlx::PgPool;
use tracing::instrument;

use crate::{
    audit::Audit,
    authorization::{Authorization, LoanAction, Object, TermAction},
    customer::Customers,
    entity::EntityError,
    job::{JobRegistry, Jobs},
    ledger::{loan::*, Ledger},
    primitives::*,
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
    customers: Customers,
    ledger: Ledger,
    pool: PgPool,
    jobs: Option<Jobs>,
    authz: Authorization,
}

impl Loans {
    pub fn new(
        pool: &PgPool,
        registry: &mut JobRegistry,
        customers: &Customers,
        ledger: &Ledger,
        authz: &Authorization,
        audit: &Audit,
    ) -> Self {
        let loan_repo = LoanRepo::new(pool);
        let term_repo = TermRepo::new(pool);
        registry.add_initializer(LoanProcessingJobInitializer::new(
            ledger,
            loan_repo.clone(),
            audit,
        ));
        Self {
            loan_repo,
            term_repo,
            customers: customers.clone(),
            ledger: ledger.clone(),
            pool: pool.clone(),
            jobs: None,
            authz: authz.clone(),
        }
    }

    pub fn set_jobs(&mut self, jobs: &Jobs) {
        self.jobs = Some(jobs.clone());
    }

    fn jobs(&self) -> &Jobs {
        self.jobs.as_ref().expect("Jobs must already be set")
    }

    pub async fn update_default_terms(
        &self,
        sub: &Subject,
        terms: TermValues,
    ) -> Result<Terms, LoanError> {
        self.authz
            .check_permission(sub, Object::Term, TermAction::Update)
            .await?;

        self.term_repo.update_default(terms).await
    }

    pub async fn create_loan_for_customer(
        &self,
        sub: &Subject,
        customer_id: impl Into<CustomerId>,
        desired_principal: UsdCents,
        terms: TermValues,
    ) -> Result<Loan, LoanError> {
        let audit_info = self
            .authz
            .check_permission(sub, Object::Loan, LoanAction::Create)
            .await?;

        let customer_id = customer_id.into();
        let customer = match self.customers.find_by_id(Some(sub), customer_id).await? {
            Some(customer) => customer,
            None => return Err(LoanError::CustomerNotFound(customer_id)),
        };

        if !customer.may_create_loan() {
            return Err(LoanError::CustomerNotAllowedToCreateLoan(customer_id));
        }
        let mut tx = self.pool.begin().await?;

        let new_loan = NewLoan::builder(audit_info)
            .id(LoanId::new())
            .customer_id(customer_id)
            .principal(desired_principal)
            .account_ids(LoanAccountIds::new())
            .terms(terms)
            .customer_account_ids(customer.account_ids)
            .build()
            .expect("could not build new loan");

        let loan = self.loan_repo.create_in_tx(&mut tx, new_loan).await?;
        self.ledger
            .create_accounts_for_loan(loan.id, loan.account_ids)
            .await?;
        tx.commit().await?;
        Ok(loan)
    }

    #[instrument(name = "lava.loan.approve_loan", skip(self), err)]
    pub async fn approve_loan(
        &self,
        sub: &Subject,
        loan_id: impl Into<LoanId> + std::fmt::Debug,
    ) -> Result<Loan, LoanError> {
        let audit_info = self
            .authz
            .check_permission(sub, Object::Loan, LoanAction::Approve)
            .await?;

        let mut loan = self.loan_repo.find_by_id(loan_id.into()).await?;
        let mut tx = self.pool.begin().await?;
        let tx_id = LedgerTxId::new();
        loan.approve(tx_id, audit_info)?;
        self.ledger
            .approve_loan(
                tx_id,
                loan.account_ids,
                loan.customer_account_ids,
                loan.initial_principal(),
                format!("{}-approval", loan.id),
            )
            .await?;
        self.loan_repo.persist_in_tx(&mut tx, &mut loan).await?;
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

    #[instrument(name = "lava.loan.update_collateral", skip(self), err)]
    pub async fn update_collateral(
        &self,
        sub: &Subject,
        loan_id: LoanId,
        updated_collateral: Satoshis,
    ) -> Result<Loan, LoanError> {
        let audit_info = self
            .authz
            .check_permission(sub, Object::Loan, LoanAction::UpdateCollateral)
            .await?;

        let mut loan = self.loan_repo.find_by_id(loan_id).await?;

        let loan_collateral_update = loan.initiate_collateral_update(updated_collateral)?;

        let mut db_tx = self.pool.begin().await?;
        let executed_at = self
            .ledger
            .update_collateral(loan_collateral_update.clone())
            .await?;

        loan.confirm_collateral_update(loan_collateral_update, executed_at, audit_info);
        self.loan_repo.persist_in_tx(&mut db_tx, &mut loan).await?;
        db_tx.commit().await?;
        Ok(loan)
    }

    pub async fn record_payment_or_complete_loan(
        &self,
        sub: &Subject,
        loan_id: LoanId,
        amount: UsdCents,
    ) -> Result<Loan, LoanError> {
        let mut db_tx = self.pool.begin().await?;

        let audit_info = self
            .authz
            .check_permission(sub, Object::Loan, LoanAction::RecordPayment)
            .await?;

        let mut loan = self.loan_repo.find_by_id(loan_id).await?;

        let customer = self.customers.repo().find_by_id(loan.customer_id).await?;
        let customer_balances = self
            .ledger
            .get_customer_balance(customer.account_ids)
            .await?;
        if customer_balances.usd_balance.settled < amount {
            return Err(LoanError::InsufficientBalance(
                customer_balances.usd_balance.settled,
                amount,
            ));
        }

        let balances = self.ledger.get_loan_balance(loan.account_ids).await?;
        assert_eq!(balances.principal_receivable, loan.outstanding().principal);
        assert_eq!(balances.interest_receivable, loan.outstanding().interest);

        let repayment = loan.initiate_repayment(amount)?;

        let executed_at = self.ledger.record_loan_repayment(repayment.clone()).await?;
        loan.confirm_repayment(repayment, executed_at, audit_info);

        self.loan_repo.persist_in_tx(&mut db_tx, &mut loan).await?;

        db_tx.commit().await?;

        Ok(loan)
    }

    pub async fn find_by_id(
        &self,
        sub: Option<&Subject>,
        id: LoanId,
    ) -> Result<Option<Loan>, LoanError> {
        if let Some(sub) = sub {
            self.authz
                .check_permission(sub, Object::Loan, LoanAction::Read)
                .await?;
        }

        match self.loan_repo.find_by_id(id).await {
            Ok(loan) => Ok(Some(loan)),
            Err(LoanError::EntityError(EntityError::NoEntityEventsPresent)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn list_for_customer(
        &self,
        sub: Option<&Subject>,
        customer_id: CustomerId,
    ) -> Result<Vec<Loan>, LoanError> {
        if let Some(sub) = sub {
            self.authz
                .check_permission(sub, Object::Loan, LoanAction::List)
                .await?;
        }

        self.loan_repo.find_for_customer(customer_id).await
    }

    pub async fn find_default_terms(&self, sub: &Subject) -> Result<Option<Terms>, LoanError> {
        self.authz
            .check_permission(sub, Object::Term, TermAction::Read)
            .await?;
        match self.term_repo.find_default().await {
            Ok(terms) => Ok(Some(terms)),
            Err(LoanError::TermsNotSet) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn list(
        &self,
        sub: &Subject,
        query: crate::query::PaginatedQueryArgs<LoanCursor>,
    ) -> Result<crate::query::PaginatedQueryRet<Loan, LoanCursor>, LoanError> {
        self.authz
            .check_permission(sub, Object::Loan, LoanAction::List)
            .await?;
        self.loan_repo.list(query).await
    }
}
