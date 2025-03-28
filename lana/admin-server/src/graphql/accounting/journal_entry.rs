use async_graphql::*;

use cala_ledger::{DebitOrCredit, Layer};
pub use lana_app::accounting::journal::JournalEntryCursor;
use lana_app::accounting::journal::{
    JournalEntry as DomainJournalEntry, JournalEntryAmount as DomainJournalEntryAmount,
};

use super::ledger_account::LedgerAccount;

use crate::{graphql::loader::LanaDataLoader, primitives::*};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct JournalEntry {
    id: ID,
    entry_id: UUID,
    tx_id: UUID,
    amount: JournalEntryAmount,
    direction: DebitOrCredit,
    layer: Layer,
    created_at: Timestamp,

    #[graphql(skip)]
    pub entity: Arc<DomainJournalEntry>,
}

impl From<DomainJournalEntry> for JournalEntry {
    fn from(entry: DomainJournalEntry) -> Self {
        Self {
            id: entry.entry_id.into(),
            entry_id: entry.entry_id.into(),
            tx_id: entry.tx_id.into(),
            amount: entry.amount.into(),
            direction: entry.direction,
            layer: entry.layer,
            created_at: entry.created_at.into(),
            entity: Arc::new(entry),
        }
    }
}

#[ComplexObject]
impl JournalEntry {
    pub async fn entry_type(&self) -> &str {
        &self.entity.entry_type
    }

    pub async fn description(&self) -> &Option<String> {
        &self.entity.description
    }

    pub async fn ledger_account(&self, ctx: &Context<'_>) -> async_graphql::Result<LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let account = loader
            .load_one(self.entity.ledger_account_id)
            .await?
            .expect("committee not found");
        Ok(account)
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
