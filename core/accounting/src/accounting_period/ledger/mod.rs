mod closing;

use chrono::NaiveDate;
use cala_ledger::{CalaLedger, account_set::AccountSetUpdate, JournalId, AccountSetId};
use super::error::AccountingPeriodError;
use closing::*;

#[derive(Clone)]
pub struct AccountingPeriodLedger {
    cala: CalaLedger,
    journal_id: JournalId,
}

impl AccountingPeriodLedger {
    pub fn new(cala: &CalaLedger, journal_id: JournalId) -> Self {
        Self {
            cala: cala.clone(),
            journal_id,
        }
    }
}

impl AccountingPeriodLedger {
    pub async fn update_close_metadata(
        &self,
        op: es_entity::DbOp<'_>,
        chart_id: impl Into<AccountSetId>,
        closed_as_of: NaiveDate,
    ) -> Result<(), AccountingPeriodError> {
        let id = chart_id.into();
        let mut chart_root_account_set = self.cala.account_sets().find(id).await?;

        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);

        let mut metadata = chart_root_account_set
            .values()
            .clone()
            .metadata
            .unwrap_or_else(|| serde_json::json!({}));
        ClosingMetadata::update_metadata(&mut metadata, closed_as_of);

        let mut update_values = AccountSetUpdate::default();
        update_values
            .metadata(Some(metadata))
            .expect("Failed to serialize metadata");

        chart_root_account_set.update(update_values);
        self.cala
            .account_sets()
            .persist_in_op(&mut op, &mut chart_root_account_set)
            .await?;
        
        op.commit().await?;
        Ok(())
    }
}
