mod entity;
pub mod error;
mod repo;

use crate::{
    entity::*,
    ledger::*,
    primitives::{Satoshis, UsdCents, UserId},
};

pub use entity::*;
use error::UserError;
pub use repo::UserRepo;

#[derive(Clone)]
pub struct Users {
    _pool: sqlx::PgPool,
    repo: UserRepo,
    ledger: Ledger,
}

impl Users {
    pub fn new(pool: &sqlx::PgPool, ledger: &Ledger) -> Self {
        let repo = UserRepo::new(pool);
        Self {
            _pool: pool.clone(),
            repo,
            ledger: ledger.clone(),
        }
    }

    pub fn repo(&self) -> &UserRepo {
        &self.repo
    }

    pub async fn create_user(&self, bitfinex_username: String) -> Result<User, UserError> {
        let id = UserId::new();
        let ledger_account_ids = self
            .ledger
            .create_accounts_for_user(&bitfinex_username)
            .await?;
        let new_user = NewUser::builder()
            .id(id)
            .bitfinex_username(bitfinex_username)
            .account_ids(ledger_account_ids)
            .build()
            .expect("Could not build User");

        let EntityUpdate { entity: user, .. } = self.repo.create(new_user).await?;
        Ok(user)
    }

    pub async fn pledge_unallocated_collateral_for_user(
        &self,
        user_id: UserId,
        amount: Satoshis,
        reference: String,
    ) -> Result<User, UserError> {
        let user = self.repo.find_by_id(user_id).await?;
        self.ledger
            .pledge_collateral_for_user(
                user.account_ids.unallocated_collateral_id,
                amount,
                reference,
            )
            .await?;
        Ok(user)
    }

    pub async fn deposit_checking_for_user(
        &self,
        user_id: UserId,
        amount: UsdCents,
        reference: String,
    ) -> Result<User, UserError> {
        let user = self.repo.find_by_id(user_id).await?;
        self.ledger
            .deposit_checking_for_user(user.account_ids.checking_id, amount, reference)
            .await?;
        Ok(user)
    }

    pub async fn find_by_id(&self, id: UserId) -> Result<Option<User>, UserError> {
        match self.repo.find_by_id(id).await {
            Ok(user) => Ok(Some(user)),
            Err(UserError::EntityError(EntityError::NoEntityEventsPresent)) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
