mod cursor;
mod entity;
mod error;
mod repo;

use crate::{
    authorization::{Authorization, DepositAction, Object},
    customer::Customers,
    data_export::Export,
    ledger::Ledger,
    primitives::{CustomerId, DepositId, Subject, UsdCents},
};

pub use cursor::*;
pub use entity::*;
use error::DepositError;
pub use repo::DepositRepo;

#[derive(Clone)]
pub struct Deposits {
    pool: sqlx::PgPool,
    repo: DepositRepo,
    customers: Customers,
    ledger: Ledger,
    authz: Authorization,
}

impl Deposits {
    pub fn new(
        pool: &sqlx::PgPool,
        customers: &Customers,
        ledger: &Ledger,
        authz: &Authorization,
        export: &Export,
    ) -> Self {
        let repo = DepositRepo::new(pool, export);
        Self {
            pool: pool.clone(),
            repo,
            customers: customers.clone(),
            ledger: ledger.clone(),
            authz: authz.clone(),
        }
    }

    pub fn repo(&self) -> &DepositRepo {
        &self.repo
    }

    pub async fn record(
        &self,
        sub: &Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
        amount: UsdCents,
        reference: Option<String>,
    ) -> Result<Deposit, DepositError> {
        let audit_id = self
            .authz
            .check_permission(sub, Object::Deposit, DepositAction::Record)
            .await?;

        let customer_id = customer_id.into();
        let customer = self.customers.repo().find_by_id(customer_id).await?;
        let new_deposit = NewDeposit::builder()
            .id(DepositId::new())
            .customer_id(customer_id)
            .amount(amount)
            .reference(reference.clone())
            .credit_account_id(customer.account_ids.on_balance_sheet_deposit_account_id)
            .audit_id(audit_id)
            .build()
            .expect("Could not build Deposit");

        let mut db_tx = self.pool.begin().await?;
        let deposit = self.repo.create_in_tx(&mut db_tx, new_deposit).await?;

        self.ledger
            .record_deposit_for_customer(
                deposit.id,
                customer.account_ids,
                amount,
                deposit.reference.clone(),
            )
            .await?;

        db_tx.commit().await?;

        Ok(deposit)
    }

    pub async fn find_by_id(
        &self,
        sub: &Subject,
        id: impl Into<DepositId> + std::fmt::Debug,
    ) -> Result<Option<Deposit>, DepositError> {
        self.authz
            .check_permission(sub, Object::Deposit, DepositAction::Read)
            .await?;

        match self.repo.find_by_id(id.into()).await {
            Ok(deposit) => Ok(Some(deposit)),
            Err(DepositError::CouldNotFindById(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn list_for_customer(
        &self,
        sub: &Subject,
        customer_id: CustomerId,
    ) -> Result<Vec<Deposit>, DepositError> {
        self.authz
            .check_permission(sub, Object::Deposit, DepositAction::List)
            .await?;

        self.repo.list_for_customer(customer_id).await
    }

    pub async fn list(
        &self,
        sub: &Subject,
        query: crate::query::PaginatedQueryArgs<DepositCursor>,
    ) -> Result<crate::query::PaginatedQueryRet<Deposit, DepositCursor>, DepositError> {
        self.authz
            .check_permission(sub, Object::Deposit, DepositAction::List)
            .await?;
        self.repo.list(query).await
    }
}
