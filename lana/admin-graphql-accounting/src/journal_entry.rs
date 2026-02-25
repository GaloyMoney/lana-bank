use async_graphql::*;

use std::sync::Arc;

use admin_graphql_shared::primitives::*;
use lana_app::primitives::{DebitOrCredit, Layer};

use lana_app::accounting::journal::{
    JournalEntry as DomainJournalEntry, JournalEntryAmount as DomainJournalEntryAmount,
};

pub use lana_app::accounting::journal::JournalEntryCursor;

use super::ledger_account::{CHART_REF, LedgerAccount};
use super::ledger_transaction::LedgerTransaction;

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

    pub async fn ledger_account(&self, ctx: &Context<'_>) -> async_graphql::Result<LedgerAccount> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let account = app
            .accounting()
            .find_ledger_account_by_id(sub, CHART_REF, self.entity.ledger_account_id)
            .await?
            .expect("ledger account not found");
        Ok(LedgerAccount::from(account))
    }

    pub async fn ledger_transaction(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<LedgerTransaction> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let tx = app
            .accounting()
            .ledger_transactions()
            .find_by_id(sub, self.entity.ledger_transaction_id)
            .await?
            .expect("ledger transaction not found");
        Ok(LedgerTransaction::from(tx))
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
