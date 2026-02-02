use obix::out::EphemeralEventType;
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::primitives::{AccountingCsvId, LedgerAccountId};

pub const CSV_EXPORT_EVENT_TYPE: EphemeralEventType =
    EphemeralEventType::new("core.accounting.csv-export");

#[derive(Debug, Clone, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreAccountingEvent {
    LedgerAccountCsvExportUploaded {
        id: AccountingCsvId,
        ledger_account_id: LedgerAccountId,
    },
}
