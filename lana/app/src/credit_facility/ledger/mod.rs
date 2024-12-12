pub mod error;
// mod templates;

use cala_ledger::{CalaLedger, JournalId};

use error::*;

#[derive(Clone)]
pub struct CreditLedger {
    _cala: CalaLedger,
    _journal_id: JournalId,
}

impl CreditLedger {
    pub async fn init(cala: &CalaLedger, journal_id: JournalId) -> Result<Self, CreditLedgerError> {
        Ok(Self {
            _cala: cala.clone(),
            _journal_id: journal_id,
        })
    }
}
