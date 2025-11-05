mod closing;
pub mod error;

use tracing::instrument;

use cala_ledger::{AccountSetId, CalaLedger, JournalId, account_set::AccountSetUpdate};

use closing::*;
use error::*;

#[derive(Clone)]
pub struct FiscalYearLedger {
    cala: CalaLedger,
    _journal_id: JournalId,
}

impl FiscalYearLedger {
    pub fn new(cala: &CalaLedger, journal_id: JournalId) -> Self {
        Self {
            cala: cala.clone(),
            _journal_id: journal_id,
        }
    }

    #[instrument(name = "fiscal_year.close_month_as_of", skip(self, op, chart_root_account_set_id), fields(chart_id = tracing::field::Empty, closed_as_of = %closed_as_of), err)]
    pub async fn close_month_as_of(
        &self,
        op: es_entity::DbOp<'_>,
        closed_as_of: chrono::NaiveDate,
        chart_root_account_set_id: impl Into<AccountSetId>,
    ) -> Result<(), FiscalYearLedgerError> {
        let id = chart_root_account_set_id.into();
        tracing::Span::current().record("chart_id", id.to_string());
        let mut chart_account_set = self.cala.account_sets().find(id).await?;

        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);

        let mut metadata = chart_account_set
            .values()
            .clone()
            .metadata
            .unwrap_or_else(|| serde_json::json!({}));
        AccountingClosingMetadata::update_metadata(&mut metadata, closed_as_of);

        let mut update_values = AccountSetUpdate::default();
        update_values
            .metadata(Some(metadata))
            .expect("Failed to serialize metadata");

        chart_account_set.update(update_values);
        self.cala
            .account_sets()
            .persist_in_op(&mut op, &mut chart_account_set)
            .await?;

        op.commit().await?;
        Ok(())
    }
}
