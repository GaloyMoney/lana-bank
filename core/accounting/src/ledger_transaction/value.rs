use std::str::FromStr;

use chrono::{DateTime, Utc};

use crate::{
    journal::JournalEntry,
    primitives::{EntityRef, LedgerTransactionId},
};

pub struct LedgerTransaction<S> {
    pub id: LedgerTransactionId,
    pub entries: Vec<JournalEntry>,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
    pub effective: chrono::NaiveDate,
    pub entity_ref: Option<EntityRef>,
    pub initiated_by: S,
}

#[derive(serde::Deserialize)]
struct ExtractMetadata {
    entity_ref: Option<EntityRef>,
    initiated_by: String,
}

impl<S>
    TryFrom<(
        cala_ledger::transaction::Transaction,
        Vec<cala_ledger::entry::Entry>,
    )> for LedgerTransaction<S>
where
    S: FromStr,
    S::Err: std::fmt::Display,
{
    type Error = super::error::LedgerTransactionError;

    fn try_from(
        (tx, entries): (
            cala_ledger::transaction::Transaction,
            Vec<cala_ledger::entry::Entry>,
        ),
    ) -> Result<Self, Self::Error> {
        let entries = entries
            .into_iter()
            .map(JournalEntry::try_from)
            .collect::<Result<_, _>>()?;

        let ExtractMetadata {
            entity_ref,
            initiated_by,
        } = tx
            .metadata::<ExtractMetadata>()?
            .expect("Could not extract metadata");

        let initiated_by = initiated_by
            .parse::<S>()
            .map_err(|e| super::error::LedgerTransactionError::InitiatedByParse(e.to_string()))?;

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
