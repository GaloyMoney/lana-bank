use async_graphql::*;

pub use lana_app::accounting::journal::LedgerEntryCursor;
use lana_app::accounting::journal::{
    LedgerEntry as DomainLedgerEntry, LedgerEntryAmount as DomainLedgerEntryAmount,
};
use lana_app::primitives::{DebitOrCredit, Layer};

use super::{ledger_account::LedgerAccount, ledger_transaction::LedgerTransaction};

use crate::{graphql::loader::LanaDataLoader, primitives::*};

#[derive(SimpleObject)]
#[graphql(
    complex,
    directive = crate::graphql::entity_key::entity_key::apply("ledgerEntryId".to_string())
)]
pub struct LedgerEntry {
    ledger_entry_id: UUID,
    amount: LedgerEntryAmount,
    direction: DebitOrCredit,
    layer: Layer,
    created_at: Timestamp,

    #[graphql(skip)]
    pub entity: Arc<DomainLedgerEntry>,
}

impl From<DomainLedgerEntry> for LedgerEntry {
    fn from(entry: DomainLedgerEntry) -> Self {
        Self {
            ledger_entry_id: entry.entry_id.into(),
            amount: entry.amount.into(),
            direction: entry.direction,
            layer: entry.layer,
            created_at: entry.created_at.into(),
            entity: Arc::new(entry),
        }
    }
}

#[ComplexObject]
impl LedgerEntry {
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
            .ok_or_else(|| Error::new("Ledger account not found"))?;
        Ok(account)
    }

    pub async fn ledger_transaction(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<LedgerTransaction> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let tx = loader
            .load_one(self.entity.ledger_transaction_id)
            .await?
            .ok_or_else(|| Error::new("Ledger transaction not found"))?;
        Ok(tx)
    }
}

#[derive(Union)]
pub enum LedgerEntryAmount {
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

impl From<DomainLedgerEntryAmount> for LedgerEntryAmount {
    fn from(amount: DomainLedgerEntryAmount) -> Self {
        match amount {
            DomainLedgerEntryAmount::Usd(usd) => LedgerEntryAmount::Usd(UsdAmount { usd }),
            DomainLedgerEntryAmount::Btc(btc) => LedgerEntryAmount::Btc(BtcAmount { btc }),
        }
    }
}
