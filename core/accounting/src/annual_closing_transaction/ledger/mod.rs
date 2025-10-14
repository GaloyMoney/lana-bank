mod template;

use template::*;
pub use template::{AnnualClosingTransactionParams, EntryParams};

use crate::primitives::CalaTxId;
use cala_ledger::CalaLedger;

use super::{ChartOfAccountsIntegrationConfig, error::AnnualClosingTransactionError};

#[derive(Clone)]
pub struct AnnualClosingTransactionLedger {
    cala: CalaLedger,
}

impl AnnualClosingTransactionLedger {
    pub fn new(cala: &CalaLedger) -> Self {
        Self { cala: cala.clone() }
    }

    pub async fn execute(
        &self,
        op: es_entity::DbOp<'_>,
        tx_id: CalaTxId,
        params: AnnualClosingTransactionParams,
    ) -> Result<(), AnnualClosingTransactionError> {
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

    pub async fn get_chart_of_accounts_integration_config(
        &self,
        reference: String,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, AnnualClosingTransactionError> {
        todo!();

        // let account_set_id = self
        //     .get_ids_from_reference(reference)
        //     .await?
        //     .account_set_id_for_config();
        //
        // let account_set = self.cala.account_sets().find(account_set_id).await?;
        // if let Some(meta) = account_set.values().metadata.as_ref() {
        //     let meta: ChartOfAccountsIntegrationMeta =
        //         serde_json::from_value(meta.clone()).expect("Could not deserialize metadata");
        //     Ok(Some(meta.config))
        // } else {
        //     Ok(None)
        // }
    }
}
