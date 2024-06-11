mod cursor;
mod entity;
pub mod error;
mod repo;

use crate::{
    entity::*,
    ledger::*,
    primitives::{LedgerAccountSetId, UserId},
};

pub use cursor::*;
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

    pub async fn create_user(
        &self,
        bitfinex_username: String,
        bfx_off_balance_sheet_integration_omnibus_account_set_id: LedgerAccountSetId,
        bfx_usdt_cash_integration_omnibus_account_set_id: LedgerAccountSetId,
    ) -> Result<User, UserError> {
        let id = UserId::new();
        let (ledger_account_ids, ledger_account_addresses) = self
            .ledger
            .create_accounts_for_user(
                &bitfinex_username,
                bfx_off_balance_sheet_integration_omnibus_account_set_id,
                bfx_usdt_cash_integration_omnibus_account_set_id,
            )
            .await?;
        let new_user = NewUser::builder()
            .id(id)
            .bitfinex_username(bitfinex_username)
            .account_ids(ledger_account_ids)
            .account_addresses(ledger_account_addresses)
            .build()
            .expect("Could not build User");

        let EntityUpdate { entity: user, .. } = self.repo.create(new_user).await?;
        Ok(user)
    }

    pub async fn find_by_id(&self, id: UserId) -> Result<Option<User>, UserError> {
        match self.repo.find_by_id(id).await {
            Ok(user) => Ok(Some(user)),
            Err(UserError::CouldNotFindById(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn list(
        &self,
        query: crate::query::PaginatedQueryArgs<UserByNameCursor>,
    ) -> Result<crate::query::PaginatedQueryRet<User, UserByNameCursor>, UserError> {
        self.repo.list(query).await
    }
}
