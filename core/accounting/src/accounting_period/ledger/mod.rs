mod closing;
mod template;

use audit::AuditInfo;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use template::*;
pub use template::{ClosingTransactionParams, EntryParams};

use super::{
    chart_of_accounts_integration::ChartOfAccountsIntegrationConfig, error::AccountingPeriodError,
};
use crate::primitives::{AccountCode, CalaTxId, ChartId};
use cala_ledger::{AccountSetId, CalaLedger, JournalId, account_set::AccountSetUpdate};
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
    pub const CHART_OF_ACCOUNTS_INTEGRATION_KEY: &'static str = "chart_of_accounts_integration";

    pub async fn get_chart_of_accounts_integration_config(
        &self,
        root_chart_account_set_id: impl Into<AccountSetId>,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, AccountingPeriodError> {
        let root_chart_account_set_id = root_chart_account_set_id.into();
        let account_set = self
            .cala
            .account_sets()
            .find(root_chart_account_set_id)
            .await?;
        if let Some(meta) = account_set.values().metadata.as_ref() {
            if let Some(chart_of_accounts_integration) =
                meta.get(Self::CHART_OF_ACCOUNTS_INTEGRATION_KEY)
            {
                let meta: ChartOfAccountsIntegrationMeta =
                    serde_json::from_value(chart_of_accounts_integration.clone())
                        .expect("could not deserialize chart_of_accounts_integration meta");
                Ok(Some(meta.config))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn attach_chart_of_accounts_integration_meta(
        &self,
        op: es_entity::DbOp<'_>,
        root_chart_account_set_id: impl Into<AccountSetId>,
        config: ChartOfAccountsIntegrationMeta,
    ) -> Result<(), AccountingPeriodError> {
        let root_chart_account_set_id = root_chart_account_set_id.into();
        let mut account_set = self
            .cala
            .account_sets()
            .find(root_chart_account_set_id)
            .await?;

        let mut metadata = account_set
            .values()
            .metadata
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));

        metadata
            .as_object_mut()
            .expect("metadata should be an object")
            .insert(
                Self::CHART_OF_ACCOUNTS_INTEGRATION_KEY.to_string(),
                serde_json::to_value(config)
                    .expect("could not serialize chart_of_accounts_integration meta"),
            );

        let mut update_values = AccountSetUpdate::default();
        update_values
            .metadata(Some(metadata))
            .expect("failed to serialize metadata");
        account_set.update(update_values);

        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);
        self.cala
            .account_sets()
            .persist_in_op(&mut op, &mut account_set)
            .await?;

        op.commit().await?;
        Ok(())
    }

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

    // TODO: Expose or make private?
    pub async fn prepare_closing_entries() -> Result<Vec<ClosingTransactionParams>, AccountingPeriodError> {
        todo!()
    }

    pub async fn execute_closing_transaction(
        &self,
        op: es_entity::DbOp<'_>,
        tx_id: CalaTxId,
        chart_id: ChartId,
        params: ClosingTransactionParams,
    ) -> Result<(), AccountingPeriodError> {
        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);
        let template =
            ClosingTransactionTemplate::init(&self.cala, params.entry_params.len()).await?;

        self.cala
            .post_transaction_in_op(&mut op, tx_id, &template.code(), params)
            .await?;

        op.commit().await?;

        Ok(())
    }
    // TODO: Refactor in AccountingPeriod model.
    pub async fn create_closing_entries(
        &self,
        id: ChartId,
    ) -> Result<Vec<ClosingTransactionParams>, AccountingPeriodError> {
        let config = self.get_chart_of_accounts_integration_config(id).await?;
        // let revenue_accounts = self.chart_ledger
        //     .find_all_accounts_by_parent_set_id(self.journal_id, revenue_set_id)
        //     .await?;

        // let expense_accounts = self.chart_ledger
        //     .find_all_accounts_by_parent_set_id(self.journal_id, expenses_set_id)
        //     .await?;

        // let cost_of_revenue_accounts = self.chart_ledger
        //     .find_all_accounts_by_parent_set_id(self.journal_id, cost_of_revenue_set_id)
        //     .await?;

        // let revenue_account_balances = self.cala
        //     .balances()
        //     .find_all(&revenue_accounts)
        //     .await?;

        // let cost_of_revenue_account_balances = self
        //     .cala
        //     .balances()
        //     .find_all(&cost_of_revenue_accounts)
        //     .await?;

        // let expenses_account_balances = self.cala
        //     .balances()
        //     .find_all(&expense_accounts)
        //     .await?;

        //let op = self.repo.begin_op().await?.with_db_time().await?;
        let entries = vec![];
        // TODO: Move logic.
        // let entries = self
        //     .chart_ledger
        //     .prepare_annual_closing_entries(
        //         op,
        //         revenue_account_balances,
        //         cost_of_revenue_account_balances,
        //         expenses_account_balances,
        //         retained_earnings_set_id,
        //         retained_losses_set_id,
        //     )
        //     .await?;

        Ok(entries)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChartOfAccountsIntegrationMeta {
    pub config: ChartOfAccountsIntegrationConfig,
    pub audit_info: AuditInfo,

    pub revenue_child_account_set_id_from_chart: AccountSetId,
    pub cost_of_revenue_child_account_set_id_from_chart: AccountSetId,
    pub expenses_child_account_set_id_from_chart: AccountSetId,
    pub equity_retained_earnings_child_account_set_id_from_chart: AccountSetId,
    pub equity_retained_losses_child_account_set_id_from_chart: AccountSetId,
}
