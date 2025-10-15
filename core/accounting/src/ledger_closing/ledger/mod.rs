mod template;

use cala_ledger::{AccountSetId, CalaLedger};

use crate::primitives::CalaTxId;

use super::error::LedgerClosingError;

use template::*;
pub use template::{AnnualClosingTransactionParams, EntryParams};

#[derive(Clone)]
pub struct ClosingLedger {
    cala: CalaLedger,
    chart_root_account_set_id: AccountSetId,
}

impl ClosingLedger {
    pub fn new(cala: &CalaLedger, chart_root_account_set_id: AccountSetId) -> Self {
        Self {
            cala: cala.clone(),
            chart_root_account_set_id,
        }
    }

    pub async fn execute_annual_closing_transaction(
        &self,
        op: es_entity::DbOp<'_>,
        tx_id: CalaTxId,
        params: AnnualClosingTransactionParams,
    ) -> Result<(), LedgerClosingError> {
        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);

        let template =
            AnnualClosingTransactionTemplate::init(&self.cala, params.entry_params.len()).await?;

        self.cala
            .post_transaction_in_op(&mut op, tx_id, &template.code(), params)
            .await?;

        op.commit().await?;

        Ok(())
    }
}
