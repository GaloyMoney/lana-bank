use serde::{Deserialize, Serialize};

use crate::primitives::{AccountingCsvId, LedgerAccountId};

#[derive(Debug, Clone, Serialize, Deserialize, strum::AsRefStr)]
#[serde(tag = "type")]
pub enum CoreAccountingEvent {
    LedgerAccountCsvExportUploaded {
        id: AccountingCsvId,
        ledger_account_id: LedgerAccountId,
    },
}
