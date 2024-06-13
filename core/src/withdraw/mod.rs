mod entity;
mod error;
mod repo;

use crate::{
    ledger::Ledger,
    primitives::{UsdCents, UserId, WithdrawId},
    user::Users,
};

pub use entity::*;
use error::WithdrawError;
pub use repo::WithdrawRepo;

#[derive(Clone)]
pub struct Withdraws {
    _pool: sqlx::PgPool,
    repo: WithdrawRepo,
    users: Users,
    ledger: Ledger,
}

impl Withdraws {
    pub fn new(pool: &sqlx::PgPool, users: &Users, ledger: &Ledger) -> Self {
        let repo = WithdrawRepo::new(pool);
        Self {
            _pool: pool.clone(),
            repo,
            users: users.clone(),
            ledger: ledger.clone(),
        }
    }

    pub fn repo(&self) -> &WithdrawRepo {
        &self.repo
    }

    pub async fn initiate(
        &self,
        user_id: impl Into<UserId> + std::fmt::Debug,
        amount: UsdCents,
        destination: String,
        reference: Option<String>,
    ) -> Result<Withdraw, WithdrawError> {
        let user_id = user_id.into();
        let user = self.users.repo().find_by_id(user_id).await?;
        let new_withdraw = NewWithdraw::builder()
            .id(WithdrawId::new())
            .user_id(user_id)
            .amount(amount)
            .reference(reference)
            .destination(destination)
            .debit_account_id(user.account_ids.checking_id)
            .build()
            .expect("Could not build Withdraw");

        let withdraw = self.repo.create(new_withdraw).await?;

        self.ledger
            .initiate_withdrawal_for_user(
                withdraw.id,
                withdraw.amount,
                withdraw.destination.clone(),
                format!("lava:withdraw:{}", withdraw.id),
                withdraw.debit_account_id,
            )
            .await?;
        Ok(withdraw)
    }
}
