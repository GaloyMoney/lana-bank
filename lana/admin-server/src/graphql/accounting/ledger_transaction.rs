use async_graphql::*;

pub use lana_app::accounting::ledger_transaction::{
    LedgerTransaction as DomainLedgerTransaction, LedgerTransactionCursor,
};

use crate::primitives::*;

use super::JournalEntry;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct LedgerTransaction {
    id: ID,
    ledger_transaction_id: UUID,
    created_at: Timestamp,
    effective: Date,
    #[graphql(skip)]
    pub entity: Arc<DomainLedgerTransaction>,
}

#[ComplexObject]
impl LedgerTransaction {
    async fn description(&self) -> &Option<String> {
        &self.entity.description
    }

    async fn entries(&self) -> Vec<JournalEntry> {
        self.entity
            .entries
            .iter()
            .map(|e| {
                let entry = e.clone();
                JournalEntry::from(entry)
            })
            .collect()
    }
}

impl From<DomainLedgerTransaction> for LedgerTransaction {
    fn from(tx: DomainLedgerTransaction) -> Self {
        Self {
            id: tx.id.to_global_id(),
            created_at: tx.created_at.into(),
            effective: tx.effective.into(),
            ledger_transaction_id: tx.id.into(),
            entity: Arc::new(tx),
        }
    }
}
