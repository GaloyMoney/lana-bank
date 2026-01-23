use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::primitives::{AccountingCsvId, LedgerAccountId};

#[derive(Debug, Clone, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreAccountingEvent {
    LedgerAccountCsvExportUploaded {
        id: AccountingCsvId,
        ledger_account_id: LedgerAccountId,
    },
}
