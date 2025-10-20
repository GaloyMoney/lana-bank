mod closing;
mod template;

use audit::AuditInfo;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rust_decimal::Decimal;

use template::*;
pub use template::{ClosingTransactionParams, EntryParams};

use super::{
    chart_of_accounts_integration::ChartOfAccountsIntegrationConfig, error::AccountingPeriodError,
};
use crate::primitives::{AccountCode, CalaTxId, ChartId, TransactionEntrySpec, CHART_OF_ACCOUNTS_ENTITY_TYPE, EntityRef, LedgerAccountId};
use cala_ledger::{AccountSetId, CalaLedger, JournalId, account_set::AccountSetUpdate, Currency, account::NewAccount, DebitOrCredit, AccountId, LedgerOperation, balance::AccountBalance};
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

    // pub fn create_closing_offset_entries(
    //     &self,
    //     revenue_accounts: HashMap<(JournalId, AccountId, Currency), AccountBalance>,
    //     expense_accounts: HashMap<(JournalId, AccountId, Currency), AccountBalance>,
    //     cost_of_revenue_accounts: HashMap<(JournalId, AccountId, Currency), AccountBalance>,
    // ) -> Result<Vec<TransactionEntrySpec>, ChartLedgerError> {
    //     let (revenue_offset_entries, net_revenue) =
    //         self._make_entries(None, revenue_accounts);
    //     let (expense_offset_entries, net_expenses) =
    //         self._make_entries(None, expense_accounts);
    //     let (cost_of_revenue_offset_entries, net_cost_of_revenue) =
    //         self._make_entries(None, cost_of_revenue_accounts);

    //     let mut all_entries = Vec::new();
    //     all_entries.extend(revenue_offset_entries);
    //     all_entries.extend(expense_offset_entries);
    //     all_entries.extend(cost_of_revenue_offset_entries);

    //     Ok(all_entries)
    // }

    pub fn create_closing_offset_entries(
        &self,
        description: Option<String>,
        accounts_by_code: HashMap<(JournalId, AccountId, Currency), AccountBalance>,
    ) -> (Vec<TransactionEntrySpec>, Decimal) {
        let mut entries = Vec::new();
        let mut net: Decimal = Decimal::from(0);
        for ((_journal_id, account_id, currency), bal_details) in accounts_by_code.iter() {
            let amt = bal_details.settled();
            if amt == Decimal::ZERO {
                continue;
            }
            net += amt;
            let direction = if bal_details.balance_type == DebitOrCredit::Debit {
                DebitOrCredit::Credit
            } else {
                DebitOrCredit::Debit
            };
            let ledger_account_id = LedgerAccountId::from(*account_id);
            let entry = TransactionEntrySpec {
                account_id: ledger_account_id,
                currency: currency.clone(),
                amount: amt,
                // TODO: Default description.
                description: description
                    .clone()
                    .unwrap_or("Annual Close Offset".to_string()),
                direction: direction,
            };
            entries.push(entry);
        }
        (entries, net)
    }

    pub async fn create_closing_equity_account_in_op(
        &self,
        op: es_entity::DbOpWithTime<'_>,
        net_earnings: Decimal,
        //id: AccountId,
        retained_earnings_account_set: AccountSetId,
        retained_losses_account_set: AccountSetId,
    ) -> Result<TransactionEntrySpec, AccountingPeriodError> {
        // TODO: Where to source ther reference, name and/or description params from?
        let (direction, parent_account_set, reference) = if net_earnings >= Decimal::ZERO {
            (
                DebitOrCredit::Credit,
                retained_earnings_account_set,
                "retained_earnings",
            )
        } else {
            (
                DebitOrCredit::Debit,
                retained_losses_account_set,
                "retained_losses",
            )
        };
        let id = AccountId::new();
        let entity_ref = EntityRef::new(CHART_OF_ACCOUNTS_ENTITY_TYPE, id);
        let mut op = self.cala.ledger_operation_from_db_op(op);
        let account_id = self
            .create_account_in_op(
                &mut op,
                id,
                reference,
                "Annual Close Net Income",
                "Annual Close Net Income",
                entity_ref,
                direction,
                parent_account_set,
            )
            .await?;
        let ledger_account_id = LedgerAccountId::from(account_id);
        Ok(TransactionEntrySpec {
            account_id: ledger_account_id,
            // TODO: Make currency a param?
            currency: Currency::USD,
            amount: net_earnings.abs(),
            description: "Annual Close Net Income to Equity".to_string(),
            direction,
        })
    }

    async fn create_account_in_op(
        &self,
        op: &mut LedgerOperation<'_>,
        id: impl Into<AccountId>,
        reference: &str,
        name: &str,
        description: &str,
        entity_ref: EntityRef,
        normal_balance_type: DebitOrCredit,
        parent_account_set: AccountSetId,
        // TODO: Metadata?
    ) -> Result<AccountId, AccountingPeriodError> {
        let id = id.into();
        let new_ledger_account = NewAccount::builder()
            .id(id)
            .external_id(reference)
            .name(name)
            .description(description)
            // TODO: Need another code parameter sourced?
            .code(id.to_string())
            .normal_balance_type(normal_balance_type)
            .metadata(serde_json::json!({"entity_ref": entity_ref}))
            .expect("Could not add metadata")
            .build()
            .expect("Could not build new account for annual close net income transfer entry");
        let ledger_account = self
            .cala
            .accounts()
            .create_in_op(op, new_ledger_account)
            .await?;
        self.cala
            .account_sets()
            .add_member_in_op(op, parent_account_set, ledger_account.id)
            .await?;

        Ok(ledger_account.id)
    }

    pub async fn execute_closing_transaction(
        &self,
        op: es_entity::DbOpWithTime<'_>,
        tx_id: CalaTxId,
        chart_id: ChartId,
        params: ClosingTransactionParams,
    ) -> Result<(), AccountingPeriodError> {
        let mut op = self
            .cala
            .ledger_operation_from_db_op(op);
        let template =
            ClosingTransactionTemplate::init(&self.cala, params.entry_params.len()).await?;

        self.cala
            .post_transaction_in_op(&mut op, tx_id, &template.code(), params)
            .await?;

        op.commit().await?;

        Ok(())
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
