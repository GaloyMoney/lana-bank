mod template;

use cala_ledger::CalaLedger;

use crate::primitives::CalaTxId;

use super::error::ManualTransactionError;

use template::*;
pub use template::{EntryParams, ManualTransactionParams};

#[derive(Clone)]
pub struct ManualTransactionLedger {
    cala: CalaLedger,
}

impl ManualTransactionLedger {
    pub fn new(cala: &CalaLedger) -> Self {
        Self { cala: cala.clone() }
    }

    pub async fn execute(
        &self,
        op: es_entity::DbOp<'_>,
        tx_id: CalaTxId,
        params: ManualTransactionParams,
    ) -> Result<(), ManualTransactionError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);

        let template =
            ManualTransactionTemplate::init(&self.cala, params.entry_params.len()).await?;

        self.cala
            .post_transaction_in_op(&mut op, tx_id, &template.code(), params)
            .await?;

        op.commit().await?;

        Ok(())
    }
}
