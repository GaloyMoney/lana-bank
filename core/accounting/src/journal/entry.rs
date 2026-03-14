use cala_ledger::entry::Entry;
use cala_ledger::{Currency, DebitOrCredit, EntryId, Layer};
use money::{Satoshis, UsdCents};
use serde::{Deserialize, Serialize};

use super::error::JournalError;
use crate::primitives::{LedgerAccountId, LedgerTransactionId};

#[derive(Clone)]
pub struct JournalEntry {
    pub ledger_account_id: LedgerAccountId,
    pub entry_id: EntryId,
    pub ledger_transaction_id: LedgerTransactionId,
    pub entry_type: String,
    pub amount: JournalEntryAmount,
    pub description: Option<String>,
    pub direction: DebitOrCredit,
    pub layer: Layer,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Copy)]
pub enum JournalEntryAmount {
    Usd(UsdCents),
    Btc(Satoshis),
}

impl TryFrom<Entry> for JournalEntry {
    type Error = JournalError;

    fn try_from(entry: Entry) -> Result<Self, Self::Error> {
        let amount = if entry.values().currency == Currency::USD {
            JournalEntryAmount::Usd(UsdCents::try_from_usd(entry.values().units)?)
        } else if entry.values().currency == Currency::BTC {
            JournalEntryAmount::Btc(Satoshis::try_from_btc(entry.values().units)?)
        } else {
            return Err(JournalError::UnexpectedCurrency);
        };
        Ok(Self {
            ledger_account_id: entry.values().account_id.into(),
            entry_id: entry.id,
            ledger_transaction_id: entry.values().transaction_id,
            entry_type: entry.values().entry_type.clone(),
            amount,
            description: entry.values().description.clone(),
            direction: entry.values().direction,
            layer: entry.values().layer,
            created_at: entry.created_at(),
        })
    }
}

impl JournalEntryAmount {
    pub fn to_display_amount(&self) -> String {
        match self {
            JournalEntryAmount::Usd(cents) => cents.to_usd().to_string(),
            JournalEntryAmount::Btc(sats) => sats.to_btc().to_string(),
        }
    }

    pub fn currency_code(&self) -> String {
        match self {
            JournalEntryAmount::Usd(_) => String::from("USD"),
            JournalEntryAmount::Btc(_) => String::from("BTC"),
        }
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
        use base64::{Engine as _, engine::general_purpose};
        let json = serde_json::to_string(&self).expect("could not serialize cursor");
        general_purpose::STANDARD_NO_PAD.encode(json.as_bytes())
    }

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        use base64::{Engine as _, engine::general_purpose};
        let bytes = general_purpose::STANDARD_NO_PAD
            .decode(s.as_bytes())
            .map_err(|e| e.to_string())?;
        let json = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }
}
