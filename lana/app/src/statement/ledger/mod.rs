pub mod error;

use cala_ledger::{CalaLedger, JournalId};

#[derive(Clone)]
pub struct StatementLedger {
    _cala: CalaLedger,
    _journal_id: JournalId,
}

impl StatementLedger {
    pub fn new(cala: &CalaLedger, journal_id: JournalId) -> Self {
        Self {
            _cala: cala.clone(),
            _journal_id: journal_id,
        }
    }
}
