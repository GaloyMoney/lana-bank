use async_graphql::*;

pub(crate) use lana_app::accounting::journal::JournalEntryCursor;
use lana_app::accounting::journal::{
    JournalEntry as DomainJournalEntry, JournalEntryAmount as DomainJournalEntryAmount,
};
use lana_app::primitives::{DebitOrCredit, Layer};

use super::{ledger_account::LedgerAccount, ledger_transaction::LedgerTransaction};

use crate::{graphql::loader::LanaDataLoader, primitives::*};

#[derive(SimpleObject)]
#[graphql(complex)]
pub(crate) struct JournalEntry {
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
            tx_id: entry.ledger_transaction_id.into(),
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

    pub(crate) async fn ledger_account(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let account = loader
            .load_one(self.entity.ledger_account_id)
            .await?
            .expect("account not found");
        Ok(account)
    }

    pub(crate) async fn ledger_transaction(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<LedgerTransaction> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let tx = loader
            .load_one(self.entity.ledger_transaction_id)
            .await?
            .expect("transaction not found");
        Ok(tx)
    }
}

#[derive(Union)]
pub(crate) enum JournalEntryAmount {
    Usd(UsdAmount),
    Btc(BtcAmount),
}

#[derive(SimpleObject)]
pub(crate) struct UsdAmount {
    usd: UsdCents,
}

#[derive(SimpleObject)]
pub(crate) struct BtcAmount {
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
