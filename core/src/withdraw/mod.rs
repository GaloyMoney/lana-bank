mod entity;
mod error;
mod repo;

use crate::{
    customer::Customers,
    ledger::Ledger,
    primitives::{CustomerId, UsdCents, WithdrawId},
};

pub use entity::*;
use error::WithdrawError;
pub use repo::WithdrawRepo;

#[derive(Clone)]
pub struct Withdraws {
    _pool: sqlx::PgPool,
    repo: WithdrawRepo,
    customers: Customers,
    ledger: Ledger,
}

impl Withdraws {
    pub fn new(pool: &sqlx::PgPool, customers: &Customers, ledger: &Ledger) -> Self {
        let repo = WithdrawRepo::new(pool);
        Self {
            _pool: pool.clone(),
            repo,
            customers: customers.clone(),
            ledger: ledger.clone(),
        }
    }

    pub fn repo(&self) -> &WithdrawRepo {
        &self.repo
    }

    pub async fn initiate(
        &self,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
        amount: UsdCents,
        reference: Option<String>,
    ) -> Result<Withdraw, WithdrawError> {
        let customer_id = customer_id.into();
        let customer = self.customers.repo().find_by_id(customer_id).await?;
        let new_withdraw = NewWithdraw::builder()
            .id(WithdrawId::new())
            .customer_id(customer_id)
            .amount(amount)
            .reference(reference)
            .debit_account_id(customer.account_ids.on_balance_sheet_deposit_account_id)
            .build()
            .expect("Could not build Withdraw");

        let withdraw = self.repo.create(new_withdraw).await?;

        self.ledger
            .initiate_withdrawal_for_customer(
                withdraw.id,
                withdraw.amount,
                format!("lava:withdraw:{}", withdraw.id),
                withdraw.debit_account_id,
            )
            .await?;
        Ok(withdraw)
    }
}
