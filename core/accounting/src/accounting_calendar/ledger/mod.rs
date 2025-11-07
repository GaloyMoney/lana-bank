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

    pub async fn monthly_close_chart_as_of(
        &self,
        op: es_entity::DbOp<'_>,
        chart_root_account_set_id: impl Into<AccountSetId>,
        closed_as_of: chrono::NaiveDate,
    ) -> Result<(), AccountingCalendarLedgerError> {
        unimplemented!()
    }
}
