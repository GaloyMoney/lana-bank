mod closing;
mod template;

use audit::AuditInfo;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use template::*;
pub use template::{ClosingTransactionParams, EntryParams};

use super::{
    chart_of_accounts_integration::ChartOfAccountsIntegrationConfig, error::AccountingPeriodError,
    primitives::ProfitAndLossClosingDetails,
};
use crate::primitives::{
    CHART_OF_ACCOUNTS_ENTITY_TYPE, CalaTxId, EntityRef, LedgerAccountId, TransactionEntrySpec,
};
use cala_ledger::{
    AccountId, AccountSetId, CalaLedger, Currency, DebitOrCredit, JournalId, LedgerOperation,
    account::NewAccount, account_set::AccountSetUpdate, balance::BalanceRange,
};
use closing::*;

#[derive(Clone)]
pub struct AccountingPeriodLedger {
    cala: CalaLedger,
}

impl AccountingPeriodLedger {
    pub fn new(cala: &CalaLedger) -> Self {
        Self { cala: cala.clone() }
    }
}

impl AccountingPeriodLedger {
    pub const CHART_OF_ACCOUNTS_INTEGRATION_KEY: &'static str = "chart_of_accounts_integration";

    pub async fn get_chart_of_accounts_integration_config(
        &self,
        root_chart_account_set_id: AccountSetId,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, AccountingPeriodError> {
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

    /// The annual closing process first creates a new equity account set member.
    /// This account is used to create the final entry of the closing transaction, which will transfer
    /// net income to a `BalanceSheet` account set member. The sole `BalanceSheet` entry in the closing
    /// transaction is then appended to `params.entry_params` (TODO: obvious review/cleanup area), prior
    /// to finally executing the closing transaction into Cala.
    pub async fn execute_closing(
        &self,
        op: es_entity::DbOp<'_>,
        net_earnings: Decimal,
        retained_earnings_account_set: AccountSetId,
        retained_losses_account_set: AccountSetId,
        ledger_tx_id: CalaTxId,
        // TODO: Create params in this function scope instead?
        mut params: ClosingTransactionParams,
    ) -> Result<LedgerAccountId, AccountingPeriodError> {
        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);
        let equity_account = self
            .create_closing_equity_account_in_op(
                &mut op,
                net_earnings,
                retained_earnings_account_set,
                retained_losses_account_set,
            )
            .await?;
        let account_id = equity_account.account_id;
        let entry_params: EntryParams = equity_account.into();
        params.entry_params.push(entry_params);

        self.execute_closing_transaction_in_op(&mut op, ledger_tx_id, params)
            .await?;
        op.commit().await?;

        Ok(account_id)
    }

    /// Creates closing offset entries for the `ProfitAndLossStatement` underlying accounts that is valid at any time during
    /// the closing grace period. Notably, this does not create the equity closing offset entry.
    pub fn create_closing_offset_entries(
        &self,
        description: Option<String>,
        period_end_balances: HashMap<(JournalId, AccountId, Currency), BalanceRange>,
    ) -> ProfitAndLossClosingDetails {
        let mut entries = Vec::new();
        let mut net: Decimal = Decimal::from(0);
        for ((_journal_id, account_id, currency), bal_details) in period_end_balances.iter() {
            let amt = bal_details.close.settled();
            net += amt;
            let direction = if bal_details.close.balance_type == DebitOrCredit::Debit {
                DebitOrCredit::Credit
            } else {
                DebitOrCredit::Debit
            };
            let ledger_account_id = LedgerAccountId::from(*account_id);
            let entry = TransactionEntrySpec {
                account_id: ledger_account_id,
                currency: currency.clone(),
                amount: amt,
                // TODO: User input or constant for entry descriptions?
                description: description
                    .clone()
                    .unwrap_or("Annual Close Offset".to_string()),
                direction: direction,
            };
            entries.push(entry);
        }
        ProfitAndLossClosingDetails {
            net_category_balance: net,
            closing_entries: entries,
        }
    }

    /// Creates a new equity account set member based on the +/- of net income (under configured account set A if loss; under configured account set B if profit).
    /// This account is used to create the final entry of the closing transaction, which will transfer
    /// net income to a `BalanceSheet` account set member.
    ///
    /// The result will be added to a Vec with existing `EntryParams` for
    /// all `ProfitAndLossStatement` accounts involved in the closing process.
    async fn create_closing_equity_account_in_op(
        &self,
        op: &mut LedgerOperation<'_>,
        net_earnings: Decimal,
        retained_earnings_account_set: AccountSetId,
        retained_losses_account_set: AccountSetId,
    ) -> Result<TransactionEntrySpec, AccountingPeriodError> {
        // TODO: Where to source ther `reference`, `name` and/or (account) `description` params from?
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
        let account_id = self
            .create_account_in_op(
                op,
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
            // TODO: Make currency a param? Need both account and entry description addressed in this scope.
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
    ) -> Result<AccountId, AccountingPeriodError> {
        let id = id.into();
        let new_ledger_account = NewAccount::builder()
            .id(id)
            .external_id(reference)
            .name(name)
            .description(description)
            // TODO: Need another `code` parameter sourced?
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

    /// Uses the `ClosingTransactionTemplate` to execute the closing transaction into Cala.
    async fn execute_closing_transaction_in_op(
        &self,
        op: &mut LedgerOperation<'_>,
        tx_id: CalaTxId,
        params: ClosingTransactionParams,
    ) -> Result<(), AccountingPeriodError> {
        let template =
            ClosingTransactionTemplate::init(&self.cala, params.entry_params.len()).await?;

        self.cala
            .post_transaction_in_op(op, tx_id, &template.code(), params)
            .await?;

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
