mod entity;
pub mod error;
mod repo;

use crate::{entity::EntityUpdate, ledger::*};

pub use entity::*;
use error::BankError;
pub use repo::BankRepo;

#[derive(Clone)]
pub struct Banks {
    _pool: sqlx::PgPool,
    repo: BankRepo,
    ledger: Ledger,
}

impl Banks {
    pub fn new(pool: &sqlx::PgPool, ledger: &Ledger) -> Self {
        let repo = BankRepo::new(pool);

        Self {
            _pool: pool.clone(),
            repo,
            ledger: ledger.clone(),
        }
    }

    pub async fn default(&self) -> Result<Bank, BankError> {
        let id = self.ledger.default_bank_id();
        if let Ok(bank) = self.repo.find_by_id(id).await {
            return Ok(bank);
        }

        let ledger_account_ids = self.ledger.create_accounts_for_bank().await?;
        let new_bank = NewBank::builder()
            .id(id)
            .account_ids(ledger_account_ids)
            .build()
            .expect("Could not build Bank");

        let err = match self.repo.create(new_bank).await {
            Ok(EntityUpdate { entity: bank, .. }) => return Ok(bank),
            Err(e) => e,
        };

        match self.repo.find_by_id(id).await {
            Ok(bank) => Ok(bank),
            Err(_) => Err(err),
        }
    }
}
