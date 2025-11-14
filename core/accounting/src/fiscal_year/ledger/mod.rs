mod closing;
pub mod error;

use tracing::instrument;

use cala_ledger::{AccountSetId, CalaLedger, account_set::AccountSetUpdate};

use closing::*;
use error::*;

#[derive(Clone)]
pub struct FiscalYearLedger {
    cala: CalaLedger,
}

impl FiscalYearLedger {
    pub fn new(cala: &CalaLedger) -> Self {
        Self { cala: cala.clone() }
    }

    #[instrument(
        name = "fiscal_year.close_month_as_of", 
        skip(self, op),
        fields(chart_id = tracing::field::Empty, closed_as_of = %closed_as_of),
        err,
    )]
    pub async fn close_month_as_of(
        &self,
        op: es_entity::DbOp<'_>,
        closed_as_of: chrono::NaiveDate,
        tracking_account_set_id: impl Into<AccountSetId> + std::fmt::Debug,
    ) -> Result<(), FiscalYearLedgerError> {
        let mut tracking_account_set = self
            .cala
            .account_sets()
            .find(tracking_account_set_id.into())
            .await?;

        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);

        let mut account_set_metadata = tracking_account_set
            .values()
            .clone()
            .metadata
            .unwrap_or_else(|| serde_json::json!({}));
        AccountingClosingMetadata::update_with_monthly_closing(
            &mut account_set_metadata,
            closed_as_of,
        );

        let mut update_values = AccountSetUpdate::default();
        update_values
            .metadata(Some(account_set_metadata))
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
