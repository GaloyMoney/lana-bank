mod template;

use audit::AuditInfo;
use serde::{Deserialize, Serialize};
use template::*;
pub use template::{AnnualClosingTransactionParams, EntryParams};

use crate::primitives::CalaTxId;
use cala_ledger::{AccountSetId, CalaLedger};

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
        root_chart_account_set_id: impl Into<AccountSetId>,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, AnnualClosingTransactionError> {
        let root_chart_account_set_id = root_chart_account_set_id.into();
        let account_set = self
            .cala
            .account_sets()
            .find(root_chart_account_set_id)
            .await?;
        if let Some(meta) = account_set.values().metadata.as_ref() {
            let meta: ChartOfAccountsIntegrationMeta =
                serde_json::from_value(meta.clone()).expect("Could not deserialize metadata");
            Ok(Some(meta.config))
        } else {
            Ok(None)
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChartOfAccountsIntegrationMeta {
    pub config: ChartOfAccountsIntegrationConfig,
    pub audit_info: AuditInfo,

    pub revenue_child_account_set_id_from_chart: AccountSetId,
    pub cost_of_revenue_child_account_set_id_from_chart: AccountSetId,
    pub expenses_child_account_set_id_from_chart: AccountSetId,
}
