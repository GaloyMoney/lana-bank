use cala_ledger::{entry::Entry, DebitOrCredit, EntryId};
use core_money::{Satoshis, UsdCents};
use serde::{Deserialize, Serialize};

use super::GeneralLedgerError;

pub enum GeneralLedgerEntry {
    Usd(UsdGeneralLedgerEntry),
    Btc(BtcGeneralLedgerEntry),
}

pub struct UsdGeneralLedgerEntry {
    pub entry_id: EntryId,
    pub entry_type: String,
    pub usd_amount: UsdCents,
    pub description: Option<String>,
    pub direction: DebitOrCredit,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct BtcGeneralLedgerEntry {
    pub entry_id: EntryId,
    pub entry_type: String,
    pub btc_amount: Satoshis,
    pub description: Option<String>,
    pub direction: DebitOrCredit,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<Entry> for GeneralLedgerEntry {
    type Error = GeneralLedgerError;

    fn try_from(entry: Entry) -> Result<Self, Self::Error> {
        if entry.values().currency == "USD".parse()? {
            Ok(Self::Usd(UsdGeneralLedgerEntry {
                entry_id: entry.id,
                entry_type: entry.values().entry_type.clone(),
                usd_amount: UsdCents::try_from_usd(entry.values().units)?,
                description: entry.values().description.clone(),
                direction: entry.values().direction,
                created_at: entry.created_at(),
            }))
        } else if entry.values().currency == "BTC".parse()? {
            Ok(Self::Btc(BtcGeneralLedgerEntry {
                entry_id: entry.id,
                entry_type: entry.values().entry_type.clone(),
                btc_amount: Satoshis::try_from_btc(entry.values().units)?,
                description: entry.values().description.clone(),
                direction: entry.values().direction,
                created_at: entry.created_at(),
            }))
        } else {
            Err(GeneralLedgerError::UnexpectedCurrency)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralLedgerEntryCursor {
    pub entry_id: EntryId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<&GeneralLedgerEntry> for GeneralLedgerEntryCursor {
    fn from(entry: &GeneralLedgerEntry) -> Self {
        match entry {
            GeneralLedgerEntry::Usd(entry) => Self {
                entry_id: entry.entry_id,
                created_at: entry.created_at,
            },
            GeneralLedgerEntry::Btc(entry) => Self {
                entry_id: entry.entry_id,
                created_at: entry.created_at,
            },
        }
    }
}

impl From<cala_ledger::entry::EntriesByCreatedAtCursor> for GeneralLedgerEntryCursor {
    fn from(cursor: cala_ledger::entry::EntriesByCreatedAtCursor) -> Self {
        Self {
            entry_id: cursor.id,
            created_at: cursor.created_at,
        }
    }
}

impl From<GeneralLedgerEntryCursor> for cala_ledger::entry::EntriesByCreatedAtCursor {
    fn from(cursor: GeneralLedgerEntryCursor) -> Self {
        Self {
            id: cursor.entry_id,
            created_at: cursor.created_at,
        }
    }
}

impl async_graphql::connection::CursorType for GeneralLedgerEntryCursor {
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
