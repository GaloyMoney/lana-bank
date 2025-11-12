pub mod error;

use tracing::instrument;

use cala_ledger::{
    CalaLedger, DebitOrCredit, JournalId, LedgerOperation, account_set::NewAccountSet,
};

use error::*;

use crate::Chart;

#[derive(Clone)]
pub struct ChartLedger {
    cala: CalaLedger,
    journal_id: JournalId,
}

impl ChartLedger {
    pub fn new(cala: &CalaLedger, journal_id: JournalId) -> Self {
        Self {
            cala: cala.clone(),
            journal_id,
        }
    }

    #[instrument(name = "chart_ledger.create_chart_root_account_set_in_op", skip(self, op, chart), fields(chart_id = %chart.id, chart_name = %chart.name), err)]
    pub async fn create_chart_root_account_set_in_op<'a>(
        &self,
        op: es_entity::DbOp<'a>,
        chart: &Chart,
    ) -> Result<LedgerOperation<'a>, ChartLedgerError> {
        let mut ledger_op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);

        let new_account_set = NewAccountSet::builder()
            .id(chart.account_set_id)
            .journal_id(self.journal_id)
            .external_id(chart.id.to_string())
            .name(chart.name.clone())
            .description(chart.name.clone())
            .normal_balance_type(DebitOrCredit::Debit)
            .build()
            .expect("Could not build new account set");

        self.cala
            .account_sets()
            .create_in_op(&mut ledger_op, new_account_set)
            .await?;

        Ok(ledger_op)
    }
}
