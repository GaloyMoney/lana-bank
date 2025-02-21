pub mod account_set;
pub mod balance;
pub mod error;
pub mod ledger;

use cala_ledger::CalaLedger;

use crate::authorization::Authorization;

pub use account_set::*;
pub use balance::*;
use error::*;
use ledger::*;

#[derive(Clone)]
pub struct Statements {
    _pool: sqlx::PgPool,
    _authz: Authorization,
    _statement_ledger: StatementLedger,
}

impl Statements {
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Authorization,
        cala: &CalaLedger,
        journal_id: cala_ledger::JournalId,
    ) -> Result<Self, StatementError> {
        let statement_ledger = StatementLedger::new(cala, journal_id);

        Ok(Self {
            _pool: pool.clone(),
            _statement_ledger: statement_ledger,
            _authz: authz.clone(),
        })
    }
}
