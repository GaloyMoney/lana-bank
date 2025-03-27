use async_graphql::*;

use cala_ledger::DebitOrCredit;
pub use lana_app::accounting::journal::JournalEntryCursor;
use lana_app::accounting::journal::{
    JournalEntry as DomainJournalEntry, JournalEntryAmount as DomainJournalEntryAmount,
};

use crate::primitives::*;

#[derive(SimpleObject)]
pub struct JournalEntry {
    id: ID,
    entry_id: UUID,
    entry_type: String,
    amount: JournalEntryAmount,
    description: Option<String>,
    direction: DebitOrCredit,
    created_at: Timestamp,
}

impl From<DomainJournalEntry> for JournalEntry {
    fn from(entry: DomainJournalEntry) -> Self {
        Self {
            id: entry.entry_id.into(),
            entry_id: entry.entry_id.into(),
            entry_type: entry.entry_type,
            amount: entry.amount.into(),
            description: entry.description,
            direction: entry.direction,
            created_at: entry.created_at.into(),
        }
    }
}

#[derive(Union)]
pub enum JournalEntryAmount {
    Usd(UsdAmount),
    Btc(BtcAmount),
}

#[derive(SimpleObject)]
pub struct UsdAmount {
    usd: UsdCents,
}

#[derive(SimpleObject)]
pub struct BtcAmount {
    btc: Satoshis,
}

impl From<DomainJournalEntryAmount> for JournalEntryAmount {
    fn from(amount: DomainJournalEntryAmount) -> Self {
        match amount {
            DomainJournalEntryAmount::Usd(usd) => JournalEntryAmount::Usd(UsdAmount { usd }),
            DomainJournalEntryAmount::Btc(btc) => JournalEntryAmount::Btc(BtcAmount { btc }),
        }
    }
}
