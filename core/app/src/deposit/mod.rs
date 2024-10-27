mod entity;
mod error;
mod repo;

use authz::PermissionCheck;

use crate::{
    audit::AuditInfo,
    authorization::{Authorization, DepositAction, Object},
    customer::Customers,
    data_export::Export,
    ledger::Ledger,
    primitives::{CustomerId, DepositId, Subject, UsdCents},
};

pub use entity::*;
use error::DepositError;
pub use repo::{cursor::DepositByCreatedAtCursor, DepositRepo};

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

    pub async fn user_can_record(
        &self,
        sub: &Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, DepositError> {
        Ok(self
            .authz
            .evaluate_permission(sub, Object::Deposit, DepositAction::Record, enforce)
            .await?)
    }

    pub async fn record(
        &self,
        sub: &Subject,
        customer_id: impl Into<CustomerId>,
        amount: UsdCents,
        reference: Option<String>,
    ) -> Result<Deposit, DepositError> {
        let audit_info = self
            .user_can_record(sub, true)
            .await?
            .expect("audit info missing");

        let customer_id = customer_id.into();
        let customer = self.customers.repo().find_by_id(customer_id).await?;
        let new_deposit = NewDeposit::builder()
            .id(DepositId::new())
            .customer_id(customer_id)
            .amount(amount)
            .reference(reference.clone())
            .credit_account_id(customer.account_ids.on_balance_sheet_deposit_account_id)
            .audit_info(audit_info)
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
            .enforce_permission(sub, Object::Deposit, DepositAction::Read)
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
            .enforce_permission(sub, Object::Deposit, DepositAction::List)
            .await?;

        Ok(self
            .repo
            .list_for_customer_id_by_created_at(customer_id, Default::default())
            .await?
            .entities)
    }

    pub async fn list(
        &self,
        sub: &Subject,
        query: es_entity::PaginatedQueryArgs<DepositByCreatedAtCursor>,
    ) -> Result<es_entity::PaginatedQueryRet<Deposit, DepositByCreatedAtCursor>, DepositError> {
        self.authz
            .enforce_permission(sub, Object::Deposit, DepositAction::List)
            .await?;
        self.repo.list_by_created_at(query).await
    }
}
