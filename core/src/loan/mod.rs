mod config;
mod entity;
pub mod error;
mod jobs;
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
    data_export::Export,
    entity::EntityError,
    job::Jobs,
    ledger::{loan::*, Ledger},
    price::Price,
    primitives::*,
};

pub use config::*;
pub use entity::*;
use error::*;
use jobs::*;
use repo::*;
pub use terms::*;

#[derive(Clone)]
pub struct Loans {
    loan_repo: LoanRepo,
    term_repo: TermRepo,
    customers: Customers,
    ledger: Ledger,
    pool: PgPool,
    jobs: Jobs,
    authz: Authorization,
    price: Price,
    config: LoanConfig,
}

impl Loans {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pool: &PgPool,
        config: LoanConfig,
        jobs: &Jobs,
        customers: &Customers,
        ledger: &Ledger,
        authz: &Authorization,
        audit: &Audit,
        export: &Export,
    ) -> Self {
        let loan_repo = LoanRepo::new(pool, export);
        let term_repo = TermRepo::new(pool);
        let price = Price::new();
        jobs.add_initializer(interest::LoanProcessingJobInitializer::new(
            ledger,
            loan_repo.clone(),
            audit,
        ));
        jobs.add_initializer(cvl::LoanProcessingJobInitializer::new(
            loan_repo.clone(),
            &price,
        ));
        Self {
            loan_repo,
            term_repo,
            customers: customers.clone(),
            ledger: ledger.clone(),
            pool: pool.clone(),
            jobs: jobs.clone(),
            authz: authz.clone(),
            price,
            config,
        }
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

    #[instrument(name = "lava.loan.create_loan_for_customer", skip(self), err)]
    pub async fn create_loan_for_customer(
        &self,
        sub: &Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
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
        let mut db_tx = self.pool.begin().await?;

        let new_loan = NewLoan::builder(audit_info)
            .id(LoanId::new())
            .customer_id(customer_id)
            .principal(desired_principal)
            .account_ids(LoanAccountIds::new())
            .terms(terms)
            .customer_account_ids(customer.account_ids)
            .build()
            .expect("could not build new loan");

        let loan = self.loan_repo.create_in_tx(&mut db_tx, new_loan).await?;
        self.ledger
            .create_accounts_for_loan(loan.id, loan.account_ids)
            .await?;
        db_tx.commit().await?;
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

        let mut db_tx = self.pool.begin().await?;
        let loan_approval = loan.initiate_approval()?;
        let executed_at = self.ledger.approve_loan(loan_approval.clone()).await?;
        loan.confirm_approval(loan_approval, executed_at, audit_info);

        self.loan_repo.persist_in_tx(&mut db_tx, &mut loan).await?;
        self.jobs
            .create_and_spawn_job::<interest::LoanProcessingJobInitializer, _>(
                &mut db_tx,
                loan.id,
                format!("loan-interest-processing-{}", loan.id),
                interest::LoanJobConfig { loan_id: loan.id },
            )
            .await?;
        db_tx.commit().await?;
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

        let price = self.price.usd_cents_per_btc().await?;

        let mut loan = self.loan_repo.find_by_id(loan_id).await?;

        let loan_collateral_update = loan.initiate_collateral_update(updated_collateral)?;

        let mut db_tx = self.pool.begin().await?;
        let executed_at = self
            .ledger
            .update_collateral(loan_collateral_update.clone())
            .await?;

        loan.confirm_collateral_update(
            loan_collateral_update,
            executed_at,
            audit_info,
            price,
            self.config.upgrade_buffer_cvl_pct,
        );
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

        let price = self.price.usd_cents_per_btc().await?;

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
        loan.confirm_repayment(
            repayment,
            executed_at,
            audit_info,
            price,
            self.config.upgrade_buffer_cvl_pct,
        );

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
