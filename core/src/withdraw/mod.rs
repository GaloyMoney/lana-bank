mod cursor;
mod entity;
mod error;
mod repo;

use crate::{
    authorization::{Authorization, Object, WithdrawAction},
    customer::Customers,
    ledger::Ledger,
    primitives::{CustomerId, Subject, UsdCents, WithdrawId},
};

pub use cursor::*;
pub use entity::*;
use error::WithdrawError;
pub use repo::WithdrawRepo;

#[derive(Clone)]
pub struct Withdraws {
    pool: sqlx::PgPool,
    repo: WithdrawRepo,
    customers: Customers,
    ledger: Ledger,
    authz: Authorization,
}

impl Withdraws {
    pub fn new(
        pool: &sqlx::PgPool,
        customers: &Customers,
        ledger: &Ledger,
        authz: &Authorization,
    ) -> Self {
        let repo = WithdrawRepo::new(pool);
        Self {
            pool: pool.clone(),
            repo,
            customers: customers.clone(),
            ledger: ledger.clone(),
            authz: authz.clone(),
        }
    }

    pub fn repo(&self) -> &WithdrawRepo {
        &self.repo
    }

    pub async fn initiate(
        &self,
        sub: &Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
        amount: UsdCents,
        reference: Option<String>,
    ) -> Result<Withdraw, WithdrawError> {
        let audit_info = self
            .authz
            .check_permission(sub, Object::Withdraw, WithdrawAction::Initiate)
            .await?;
        let customer_id = customer_id.into();
        let customer = self.customers.repo().find_by_id(customer_id).await?;
        let new_withdraw = NewWithdraw::builder()
            .id(WithdrawId::new())
            .customer_id(customer_id)
            .amount(amount)
            .reference(reference)
            .debit_account_id(customer.account_ids.on_balance_sheet_deposit_account_id)
            .audit_info(audit_info)
            .build()
            .expect("Could not build Withdraw");

        let mut db_tx = self.pool.begin().await?;
        let withdraw = self.repo.create_in_tx(&mut db_tx, new_withdraw).await?;

        let customer_balances = self
            .ledger
            .get_customer_balance(customer.account_ids)
            .await?;
        if customer_balances.usd_balance.settled < amount {
            return Err(WithdrawError::InsufficientBalance(
                customer_balances.usd_balance.settled,
                amount,
            ));
        }

        self.ledger
            .initiate_withdrawal_for_customer(
                withdraw.id,
                customer.account_ids,
                withdraw.amount,
                format!("lava:withdraw:{}:initiate", withdraw.id),
            )
            .await?;

        db_tx.commit().await?;

        Ok(withdraw)
    }

    pub async fn confirm(
        &self,
        sub: &Subject,
        withdrawal_id: impl Into<WithdrawId> + std::fmt::Debug,
    ) -> Result<Withdraw, WithdrawError> {
        let audit_info = self
            .authz
            .check_permission(sub, Object::Withdraw, WithdrawAction::Confirm)
            .await?;
        let id = withdrawal_id.into();
        let mut withdrawal = self.repo.find_by_id(id).await?;
        let tx_id = withdrawal.confirm(audit_info)?;

        let mut db_tx = self.pool.begin().await?;
        self.repo.persist_in_tx(&mut db_tx, &mut withdrawal).await?;

        self.ledger
            .confirm_withdrawal_for_customer(
                tx_id,
                withdrawal.id,
                withdrawal.debit_account_id,
                withdrawal.amount,
                format!("lava:withdraw:{}:confirm", withdrawal.id),
            )
            .await?;

        db_tx.commit().await?;

        Ok(withdrawal)
    }

    pub async fn cancel(
        &self,
        sub: &Subject,
        withdrawal_id: impl Into<WithdrawId> + std::fmt::Debug,
    ) -> Result<Withdraw, WithdrawError> {
        let audit_info = self
            .authz
            .check_permission(sub, Object::Withdraw, WithdrawAction::Cancel)
            .await?;

        let id = withdrawal_id.into();
        let mut withdrawal = self.repo.find_by_id(id).await?;
        let tx_id = withdrawal.cancel(audit_info)?;

        let mut db_tx = self.pool.begin().await?;
        self.repo.persist_in_tx(&mut db_tx, &mut withdrawal).await?;

        self.ledger
            .cancel_withdrawal_for_customer(
                tx_id,
                withdrawal.id,
                withdrawal.debit_account_id,
                withdrawal.amount,
                format!("lava:withdraw:{}:cancel", withdrawal.id),
            )
            .await?;

        db_tx.commit().await?;

        Ok(withdrawal)
    }

    pub async fn find_by_id(
        &self,
        sub: &Subject,
        id: impl Into<WithdrawId> + std::fmt::Debug,
    ) -> Result<Option<Withdraw>, WithdrawError> {
        self.authz
            .check_permission(sub, Object::Withdraw, WithdrawAction::Read)
            .await?;

        match self.repo.find_by_id(id.into()).await {
            Ok(withdrawal) => Ok(Some(withdrawal)),
            Err(WithdrawError::CouldNotFindById(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn list_for_customer(
        &self,
        sub: &Subject,
        customer_id: CustomerId,
    ) -> Result<Vec<Withdraw>, WithdrawError> {
        self.authz
            .check_permission(sub, Object::Withdraw, WithdrawAction::List)
            .await?;

        self.repo.list_for_customer(customer_id).await
    }

    pub async fn list(
        &self,
        sub: &Subject,
        query: crate::query::PaginatedQueryArgs<WithdrawCursor>,
    ) -> Result<crate::query::PaginatedQueryRet<Withdraw, WithdrawCursor>, WithdrawError> {
        self.authz
            .check_permission(sub, Object::Withdraw, WithdrawAction::List)
            .await?;
        self.repo.list(query).await
    }
}
