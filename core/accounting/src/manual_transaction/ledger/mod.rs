pub mod error;
mod template;

use cala_ledger::CalaLedger;

use crate::primitives::CalaTxId;

use error::*;
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

    pub async fn execute_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        tx_id: CalaTxId,
        params: ManualTransactionParams,
    ) -> Result<(), ManualTransactionLedgerError> {
        let template =
            ManualTransactionTemplate::init(&self.cala, params.entry_params.len()).await?;

        self.cala
            .post_transaction_in_op(op, tx_id, &template.code(), params)
            .await?;

        Ok(())
    }
}
