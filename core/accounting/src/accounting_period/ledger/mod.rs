mod closing;

use chrono::NaiveDate;

use cala_ledger::{AccountSetId, CalaLedger, account_set::AccountSetUpdate};

use super::error::AccountingPeriodError;

use closing::*;

#[derive(Clone)]
pub struct AccountingPeriodLedger {
    cala: CalaLedger,
}

impl AccountingPeriodLedger {
    pub fn new(cala: &CalaLedger) -> Self {
        Self { cala: cala.clone() }
    }
}

impl AccountingPeriodLedger {
    pub async fn update_close_metadata_in_op(
        &self,
        op: es_entity::DbOp<'_>,
        tracking_account_set_id: AccountSetId,
        closed_as_of: NaiveDate,
    ) -> Result<(), AccountingPeriodError> {
        let mut tracking_account_set = self
            .cala
            .account_sets()
            .find(tracking_account_set_id)
            .await?;

        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);

        let mut metadata = tracking_account_set
            .values()
            .clone()
            .metadata
            .unwrap_or_else(|| serde_json::json!({}));
        ClosingMetadata::update_metadata(&mut metadata, closed_as_of);

        let mut update_values = AccountSetUpdate::default();
        update_values
            .metadata(Some(metadata))
            .expect("Failed to serialize metadata");

        tracking_account_set.update(update_values);
        self.cala
            .account_sets()
            .persist_in_op(&mut op, &mut tracking_account_set)
            .await?;

        op.commit().await?;
        Ok(())
    }
}
