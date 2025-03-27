use cala_ledger::{entry::Entry, AccountId, DebitOrCredit, EntryId};
use core_money::{Satoshis, UsdCents};
use serde::{Deserialize, Serialize};

use super::error::JournalError;

pub struct JournalEntry {
    pub ledger_account_id: AccountId,
    pub entry_id: EntryId,
    pub entry_type: String,
    pub amount: JournalEntryAmount,
    pub description: Option<String>,
    pub direction: DebitOrCredit,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub enum JournalEntryAmount {
    Usd(UsdCents),
    Btc(Satoshis),
}

impl TryFrom<Entry> for JournalEntry {
    type Error = JournalError;

    fn try_from(entry: Entry) -> Result<Self, Self::Error> {
        let amount = if entry.values().currency == "USD".parse().expect("parse USD") {
            JournalEntryAmount::Usd(UsdCents::try_from_usd(entry.values().units)?)
        } else if entry.values().currency == "BTC".parse().expect("parse BTC") {
            JournalEntryAmount::Btc(Satoshis::try_from_btc(entry.values().units)?)
        } else {
            return Err(JournalError::UnexpectedCurrency);
        };
        Ok(Self {
            ledger_account_id: entry.values().account_id,
            entry_id: entry.id,
            entry_type: entry.values().entry_type.clone(),
            amount,
            description: entry.values().description.clone(),
            direction: entry.values().direction,
            created_at: entry.created_at(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntryCursor {
    pub entry_id: EntryId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<&JournalEntry> for JournalEntryCursor {
    fn from(entry: &JournalEntry) -> Self {
        Self {
            entry_id: entry.entry_id,
            created_at: entry.created_at,
        }
    }
}

impl From<cala_ledger::entry::EntriesByCreatedAtCursor> for JournalEntryCursor {
    fn from(cursor: cala_ledger::entry::EntriesByCreatedAtCursor) -> Self {
        Self {
            entry_id: cursor.id,
            created_at: cursor.created_at,
        }
    }
}

impl From<JournalEntryCursor> for cala_ledger::entry::EntriesByCreatedAtCursor {
    fn from(cursor: JournalEntryCursor) -> Self {
        Self {
            id: cursor.entry_id,
            created_at: cursor.created_at,
        }
    }
}

#[cfg(feature = "graphql")]
impl async_graphql::connection::CursorType for JournalEntryCursor {
    type Error = String;

    fn encode_cursor(&self) -> String {
        use base64::{engine::general_purpose, Engine as _};
        let json = serde_json::to_string(&self).expect("could not serialize cursor");
        general_purpose::STANDARD_NO_PAD.encode(json.as_bytes())
    }

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        use base64::{engine::general_purpose, Engine as _};
        let bytes = general_purpose::STANDARD_NO_PAD
            .decode(s.as_bytes())
            .map_err(|e| e.to_string())?;
        let json = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }
}
