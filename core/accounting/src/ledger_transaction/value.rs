use chrono::{DateTime, Utc};

use crate::{
    journal::JournalEntry,
    primitives::{EntityRef, LedgerTransactionId},
};

pub struct LedgerTransaction {
    pub id: LedgerTransactionId,
    pub entries: Vec<JournalEntry>,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
    pub effective: chrono::NaiveDate,
    pub entity_ref: Option<EntityRef>,
}

#[derive(serde::Deserialize)]
struct ExtractEntityRef {
    entity_ref: Option<EntityRef>,
}
impl
    TryFrom<(
        cala_ledger::transaction::Transaction,
        Vec<cala_ledger::entry::Entry>,
    )> for LedgerTransaction
{
    type Error = super::error::LedgerTransactionError;

    fn try_from(
        (tx, entries): (
            cala_ledger::transaction::Transaction,
            Vec<cala_ledger::entry::Entry>,
        ),
    ) -> Result<Self, super::error::LedgerTransactionError> {
        let entries = entries
            .into_iter()
            .map(JournalEntry::try_from)
            .collect::<Result<_, _>>()?;

        let extracted = tx
            .metadata::<ExtractEntityRef>()
            .expect("Could not extract entity_ref");

        Ok(Self {
            id: tx.id,
            entity_ref: extracted.and_then(|e| e.entity_ref),
            entries,
            created_at: tx.created_at(),
            effective: tx.effective(),
            description: tx.into_values().description,
        })
    }
}
