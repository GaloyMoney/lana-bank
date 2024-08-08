mod entity;
mod error;
mod repo;

use crate::{
    authorization::{Action, Authorization, DepositAction, Object},
    customer::Customers,
    ledger::Ledger,
    primitives::{CustomerId, DepositId, Subject, UsdCents},
};

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
    ) -> Self {
        let repo = DepositRepo::new(pool);
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
        self.authz
            .check_permission(sub, Object::Deposit, DepositAction::Record)
            .await?;

        let customer_id = customer_id.into();
        let customer = self.customers.repo().find_by_id(customer_id).await?;
        let new_deposit = NewDeposit::builder()
            .id(DepositId::new())
            .customer_id(customer_id)
            .amount(amount)
            .reference(reference)
            .credit_account_id(customer.account_ids.on_balance_sheet_deposit_account_id)
            .build()
            .expect("Could not build Deposit");

        let mut db_tx = self.pool.begin().await?;
        let deposit = self.repo.create_in_tx(&mut db_tx, new_deposit).await?;

        // self.ledger.record_deposit(deposit).await?;

        db_tx.commit().await?;

        Ok(deposit)
    }
}
