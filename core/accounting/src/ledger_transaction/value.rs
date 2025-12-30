use chrono::{DateTime, Utc};

use crate::{
    journal::JournalEntry,
    primitives::{EntityRef, LedgerTransactionId, LedgerTransactionInitiator},
};

pub struct LedgerTransaction {
    pub id: LedgerTransactionId,
    pub entries: Vec<JournalEntry>,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
    pub effective: chrono::NaiveDate,
    pub entity_ref: Option<EntityRef>,
    pub initiated_by: Option<LedgerTransactionInitiator>,
}

#[derive(serde::Deserialize)]
struct ExtractMetadata {
    entity_ref: Option<EntityRef>,
    initiated_by: Option<String>,
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

        let extracted = tx.metadata::<ExtractMetadata>().unwrap_or(None);
        let (entity_ref, initiated_by) = match extracted {
            Some(meta) => {
                let initiated_by = meta
                    .initiated_by
                    .as_deref()
                    .and_then(|raw| raw.parse::<LedgerTransactionInitiator>().ok());
                (meta.entity_ref, initiated_by)
            }
            None => (None, None),
        };

        Ok(Self {
            id: tx.id,
            entity_ref,
            initiated_by,
            entries,
            created_at: tx.created_at(),
            effective: tx.effective(),
            description: tx.into_values().description,
        })
    }
}
