mod config;
mod entity;
pub mod error;
mod history;
mod jobs;
mod repayment_plan;
mod repo;

use sqlx::PgPool;
use tracing::instrument;

use std::collections::HashMap;

use authz::PermissionCheck;

use crate::{
    audit::{Audit, AuditInfo},
    authorization::{Authorization, CustomerAllOrOne, LoanAction, LoanAllOrOne, Object},
    customer::Customers,
    data_export::Export,
    job::Jobs,
    ledger::{loan::*, Ledger},
    price::Price,
    primitives::*,
    terms::*,
};

pub use config::*;
pub use entity::*;
use error::*;
pub use history::*;
use jobs::*;
pub use repayment_plan::*;
pub use repo::{cursor::*, LoanRepo};

#[derive(Clone)]
pub struct Loans {
    loan_repo: LoanRepo,
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
    pub async fn init(
        pool: &PgPool,
        config: LoanConfig,
        jobs: &Jobs,
        customers: &Customers,
        ledger: &Ledger,
        authz: &Authorization,
        audit: &Audit,
        export: &Export,
        price: &Price,
    ) -> Result<Self, LoanError> {
        let loan_repo = LoanRepo::new(pool, export);
        jobs.add_initializer(interest::LoanProcessingJobInitializer::new(
            ledger,
            loan_repo.clone(),
            audit,
        ));
        jobs.add_initializer_and_spawn_unique(
            cvl::LoanProcessingJobInitializer::new(loan_repo.clone(), price, audit),
            cvl::LoanJobConfig {
                job_interval: std::time::Duration::from_secs(30),
                upgrade_buffer_cvl_pct: config.upgrade_buffer_cvl_pct,
            },
        )
        .await?;
        Ok(Self {
            loan_repo,
            customers: customers.clone(),
            ledger: ledger.clone(),
            pool: pool.clone(),
            jobs: jobs.clone(),
            authz: authz.clone(),
            price: price.clone(),
            config,
        })
    }

    pub async fn user_can_create_loan_for_customer(
        &self,
        sub: &Subject,
        customer_id: CustomerId,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, LoanError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                Object::Customer(CustomerAllOrOne::ById(customer_id)),
                LoanAction::Create,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "loan.create_loan_for_customer", skip(self), err)]
    pub async fn create_loan_for_customer(
        &self,
        sub: &Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
        desired_principal: UsdCents,
        terms: TermValues,
    ) -> Result<Loan, LoanError> {
        let customer_id = customer_id.into();

        let audit_info = self
            .user_can_create_loan_for_customer(sub, customer_id, true)
            .await?
            .expect("audit info missing");

        let customer = match self.customers.find_by_id(sub, customer_id).await? {
            Some(customer) => customer,
            None => return Err(LoanError::CustomerNotFound(customer_id)),
        };

        if !customer.may_create_loan() {
            return Err(LoanError::CustomerNotAllowedToCreateLoan(customer_id));
        }
        let mut db_tx = self.pool.begin().await?;

        let new_loan = NewLoan::builder()
            .id(LoanId::new())
            .audit_info(audit_info)
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

    pub async fn user_can_approve(
        &self,
        sub: &Subject,
        loan_id: LoanId,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, LoanError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                Object::Loan(LoanAllOrOne::ById(loan_id)),
                LoanAction::Approve,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "loan.add_approval", skip(self), err)]
    pub async fn add_approval(
        &self,
        sub: &Subject,
        loan_id: impl Into<LoanId> + std::fmt::Debug,
    ) -> Result<Loan, LoanError> {
        let loan_id = loan_id.into();

        let audit_info = self
            .user_can_approve(sub, loan_id, true)
            .await?
            .expect("audit info missing");

        let mut loan = self.loan_repo.find_by_id(loan_id).await?;

        let mut db_tx = self.pool.begin().await?;
        let price = self.price.usd_cents_per_btc().await?;

        let user_id = UserId::try_from(sub).map_err(|_| LoanError::SubjectIsNotUser)?;
        if let Some(loan_approval) = loan.add_approval(user_id, audit_info.clone(), price)? {
            let executed_at = self.ledger.approve_loan(loan_approval.clone()).await?;
            loan.confirm_approval(loan_approval, executed_at, audit_info);
            self.jobs
                .create_and_spawn_in_tx(
                    &mut db_tx,
                    loan.id,
                    interest::LoanJobConfig { loan_id: loan.id },
                )
                .await?;
        }
        self.loan_repo.update_in_tx(&mut db_tx, &mut loan).await?;
        db_tx.commit().await?;

        Ok(loan)
    }

    pub async fn user_can_update_collateral(
        &self,
        sub: &Subject,
        loan_id: LoanId,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, LoanError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                Object::Loan(LoanAllOrOne::ById(loan_id)),
                LoanAction::UpdateCollateral,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "loan.update_collateral", skip(self), err)]
    pub async fn update_collateral(
        &self,
        sub: &Subject,
        loan_id: LoanId,
        updated_collateral: Satoshis,
    ) -> Result<Loan, LoanError> {
        let audit_info = self
            .user_can_update_collateral(sub, loan_id, true)
            .await?
            .expect("audit info missing");

        let price = self.price.usd_cents_per_btc().await?;

        let mut loan = self.loan_repo.find_by_id(loan_id).await?;

        let loan_collateral_update = loan.initiate_collateral_update(updated_collateral)?;

        let mut db_tx = self.pool.begin().await?;
        let executed_at = self
            .ledger
            .update_loan_collateral(loan_collateral_update.clone())
            .await?;

        loan.confirm_collateral_update(
            loan_collateral_update,
            executed_at,
            audit_info,
            price,
            self.config.upgrade_buffer_cvl_pct,
        );
        self.loan_repo.update_in_tx(&mut db_tx, &mut loan).await?;
        db_tx.commit().await?;
        Ok(loan)
    }

    pub async fn subject_can_update_collateralization_state(
        &self,
        sub: &Subject,
        loan_id: LoanId,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, LoanError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                Object::Loan(LoanAllOrOne::ById(loan_id)),
                LoanAction::UpdateCollateralizationState,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "loan.trigger_collateralization_state_refresh", skip(self), err)]
    pub async fn trigger_collateralization_state_refresh(
        &self,
        sub: &Subject,
        loan_id: impl Into<LoanId> + std::fmt::Debug,
    ) -> Result<Loan, LoanError> {
        let loan_id = loan_id.into();
        let audit_info = self
            .subject_can_update_collateralization_state(sub, loan_id, true)
            .await?
            .expect("audit info missing");

        let price = self.price.usd_cents_per_btc().await?;

        let mut loan = self.loan_repo.find_by_id(loan_id).await?;

        if loan
            .maybe_update_collateralization_with_liquidation_override(
                price,
                self.config.upgrade_buffer_cvl_pct,
                &audit_info,
            )
            .is_some()
        {
            self.loan_repo.update(&mut loan).await?;
        }

        Ok(loan)
    }

    pub async fn user_can_record_payment_or_complete_loan(
        &self,
        sub: &Subject,
        loan_id: LoanId,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, LoanError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                Object::Loan(LoanAllOrOne::ById(loan_id)),
                LoanAction::RecordPayment,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "loan.record_payment_or_complete_loan", skip(self), err)]
    pub async fn record_payment_or_complete_loan(
        &self,
        sub: &Subject,
        loan_id: impl Into<LoanId> + std::fmt::Debug,
        amount: UsdCents,
    ) -> Result<Loan, LoanError> {
        let loan_id = loan_id.into();

        let mut db_tx = self.pool.begin().await?;

        let audit_info = self
            .user_can_record_payment_or_complete_loan(sub, loan_id, true)
            .await?
            .expect("audit info missing");

        let price = self.price.usd_cents_per_btc().await?;

        let mut loan = self.loan_repo.find_by_id(loan_id).await?;

        let customer = self.customers.repo().find_by_id(loan.customer_id).await?;
        self.ledger
            .get_customer_balance(customer.account_ids)
            .await?
            .check_withdraw_amount(amount)?;

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

        self.loan_repo.update_in_tx(&mut db_tx, &mut loan).await?;

        db_tx.commit().await?;

        Ok(loan)
    }

    #[instrument(name = "loan.find_by_id", skip(self), err)]
    pub async fn find_by_id(
        &self,
        sub: &Subject,
        id: impl Into<LoanId> + std::fmt::Debug,
    ) -> Result<Option<Loan>, LoanError> {
        let id = id.into();
        self.authz
            .enforce_permission(sub, Object::Loan(LoanAllOrOne::ById(id)), LoanAction::Read)
            .await?;

        match self.loan_repo.find_by_id(id).await {
            Ok(loan) => Ok(Some(loan)),
            Err(LoanError::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }

    #[instrument(name = "loan.list_for_customer", skip(self), err)]
    pub async fn list_for_customer(
        &self,
        sub: Option<&Subject>,
        customer_id: CustomerId,
    ) -> Result<Vec<Loan>, LoanError> {
        if let Some(sub) = sub {
            self.authz
                .enforce_permission(
                    sub,
                    Object::Customer(CustomerAllOrOne::ById(customer_id)),
                    LoanAction::List,
                )
                .await?;
        }

        Ok(self
            .loan_repo
            .list_for_customer_id_by_created_at(
                customer_id,
                Default::default(),
                es_entity::ListDirection::Descending,
            )
            .await?
            .entities)
    }

    #[instrument(name = "loan.list", skip(self), err)]
    pub async fn list(
        &self,
        sub: &Subject,
        query: es_entity::PaginatedQueryArgs<LoanByCreatedAtCursor>,
    ) -> Result<es_entity::PaginatedQueryRet<Loan, LoanByCreatedAtCursor>, LoanError> {
        self.authz
            .enforce_permission(sub, Object::Loan(LoanAllOrOne::All), LoanAction::List)
            .await?;
        self.loan_repo
            .list_by_created_at(query, es_entity::ListDirection::Descending)
            .await
    }

    #[instrument(name = "loan.list_by_collateralization_ratio", skip(self), err)]
    pub async fn list_by_collateralization_ratio(
        &self,
        sub: &Subject,
        query: es_entity::PaginatedQueryArgs<LoanByCollateralizationRatioCursor>,
    ) -> Result<es_entity::PaginatedQueryRet<Loan, LoanByCollateralizationRatioCursor>, LoanError>
    {
        self.authz
            .enforce_permission(sub, Object::Loan(LoanAllOrOne::All), LoanAction::List)
            .await?;
        self.loan_repo
            .list_by_collateralization_ratio(query, es_entity::ListDirection::Ascending)
            .await
    }

    pub async fn find_all<T: From<Loan>>(
        &self,
        ids: &[LoanId],
    ) -> Result<HashMap<LoanId, T>, LoanError> {
        self.loan_repo.find_all(ids).await
    }
}
