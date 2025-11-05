pub mod error;

use cala_ledger::{AccountSetId, CalaLedger};

use error::*;

#[derive(Clone)]
pub struct AccountingCalendarLedger {
    cala: CalaLedger,
}

impl AccountingCalendarLedger {
    pub fn new(cala: &CalaLedger) -> Self {
        Self { cala: cala.clone() }
    }
}
