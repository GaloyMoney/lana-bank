mod closing;
mod error;
mod template;

use audit::{AuditInfo, PaginatedQueryArgs, PaginatedQueryRet};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub use error::AccountingPeriodLedgerError;
pub use template::ClosingTransactionParams;
use template::*;

use super::{
    AccountingPeriod,
    chart_of_accounts_integration::ChartOfAccountsIntegrationConfig,
    closing::{ClosingAccountBalances, ClosingAccountEntry},
};
use crate::primitives::CalaTxId;
use cala_ledger::{
    AccountId, AccountSetId, BalanceId, CalaLedger, Currency, DebitOrCredit, JournalId,
    LedgerOperation,
    account::{Account, NewAccount},
    account_set::{AccountSetMemberId, AccountSetUpdate},
};
pub(crate) use closing::ClosingMetadata;

/// Collection of account set ID's relevant for an accounting period.
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct AccountingPeriodAccountSetIds {
    pub tracking_account_set_id: AccountSetId,
    pub revenue_account_set_id: AccountSetId,
    pub cost_of_revenue_account_set_id: AccountSetId,
    pub expenses_account_set_id: AccountSetId,
    pub equity_retained_earnings_account_set_id: AccountSetId,
    pub equity_retained_losses_account_set_id: AccountSetId,
}

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
    pub async fn close_year_in_op(
        &self,
        db: es_entity::DbOp<'_>,
        ledger_tx_id: CalaTxId,
        description: Option<String>,
        accounting_period: AccountingPeriod,
    ) -> Result<(), AccountingPeriodLedgerError> {
        let mut db = self
            .cala
            .ledger_operation_from_db_op(db.with_db_time().await?);

        self.post_closing_transaction_in_op(&mut db, ledger_tx_id, description, &accounting_period)
            .await?;

        self.update_close_metadata_in_ledger_op(
            &mut db,
            accounting_period.account_set_ids.tracking_account_set_id,
            accounting_period.period_end(),
        )
        .await?;

        db.commit().await?;

        Ok(())
    }

    async fn post_closing_transaction_in_op(
        &self,
        db: &mut LedgerOperation<'_>,
        ledger_tx_id: CalaTxId,
        description: Option<String>,
        accounting_period: &AccountingPeriod,
    ) -> Result<(), AccountingPeriodLedgerError> {
        let (net_income, mut closing_entries) = self
            .get_closing_account_entry_params(accounting_period)
            .await?
            .to_closing_entries();

        let equity_entry = self
            .create_equity_entry(db, accounting_period, net_income)
            .await?;
        closing_entries.push(equity_entry);

        let closing_transaction_params = ClosingTransactionParams::new(
            self.journal_id,
            description.unwrap_or(format!("Annual Closing {accounting_period}",)),
            accounting_period.period_end(),
            closing_entries,
        );

        let template = ClosingTransactionTemplate::init(
            &self.cala,
            closing_transaction_params.closing_entries.len(),
            accounting_period.to_string(),
        )
        .await?;

        self.cala
            .post_transaction_in_op(
                db,
                ledger_tx_id,
                &template.code(),
                closing_transaction_params,
            )
            .await?;

        Ok(())
    }

    pub async fn update_close_metadata_in_op(
        &self,
        db: es_entity::DbOp<'_>,
        tracking_account_set_id: AccountSetId,
        closed_as_of: NaiveDate,
    ) -> Result<(), AccountingPeriodLedgerError> {
        let mut op = self
            .cala
            .ledger_operation_from_db_op(db.with_db_time().await?);

        self.update_close_metadata_in_ledger_op(&mut op, tracking_account_set_id, closed_as_of)
            .await?;

        op.commit().await?;

        Ok(())
    }

    async fn update_close_metadata_in_ledger_op(
        &self,
        db: &mut LedgerOperation<'_>,
        tracking_account_set_id: AccountSetId,
        closed_as_of: NaiveDate,
    ) -> Result<(), AccountingPeriodLedgerError> {
        let mut tracking_account_set = self
            .cala
            .account_sets()
            .find(tracking_account_set_id)
            .await?;

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
            .persist_in_op(db, &mut tracking_account_set)
            .await?;

        Ok(())
    }

    /// Collects `BalanceRange` for all underlying accounts and nested underlying accounts.
    /// using `period_end` from the `AccountingPeriod` entity is used to get the effective
    /// balance from cala at that time.
    ///
    /// This amount is used to create the offset/closing  entry for the
    /// `ProfitAndLossStatement` account that is valid at any time during
    /// the closing grace period.
    pub async fn get_closing_account_entry_params(
        &self,
        period: &AccountingPeriod,
    ) -> Result<ClosingAccountBalances, AccountingPeriodLedgerError> {
        let revenue_accounts = self
            .find_all_accounts_by_parent_set_id(period.account_set_ids.revenue_account_set_id)
            .await?;
        let expense_accounts = self
            .find_all_accounts_by_parent_set_id(period.account_set_ids.expenses_account_set_id)
            .await?;
        let cost_of_revenue_accounts = self
            .find_all_accounts_by_parent_set_id(
                period.account_set_ids.cost_of_revenue_account_set_id,
            )
            .await?;

        let from = period.period_start();
        let until = period.period_end();

        let revenue_account_balances = self
            .cala
            .balances()
            .effective()
            .find_all_in_range(&revenue_accounts, from, Some(until))
            .await?;
        let cost_of_revenue_account_balances = self
            .cala
            .balances()
            .effective()
            .find_all_in_range(&cost_of_revenue_accounts, from, Some(until))
            .await?;
        let expenses_account_balances = self
            .cala
            .balances()
            .effective()
            .find_all_in_range(&expense_accounts, from, Some(until))
            .await?;

        Ok(ClosingAccountBalances {
            revenue: revenue_account_balances,
            cost_of_revenue: cost_of_revenue_account_balances,
            expenses: expenses_account_balances,
        })
    }

    async fn find_all_accounts_by_parent_set_id(
        &self,
        parent_set_id: AccountSetId,
    ) -> Result<Vec<BalanceId>, AccountingPeriodLedgerError> {
        let mut accounts: Vec<BalanceId> = Vec::new();

        let mut has_next_page = true;
        let mut after = None;

        while has_next_page {
            let PaginatedQueryRet {
                entities,
                has_next_page: next_page,
                end_cursor,
            } = self
                .cala
                .account_sets()
                .list_members_by_created_at(parent_set_id, PaginatedQueryArgs { first: 100, after })
                .await?;

            after = end_cursor;
            has_next_page = next_page;

            for member in entities {
                match member.id {
                    AccountSetMemberId::Account(account_id) => {
                        accounts.push((self.journal_id, account_id, Currency::USD));
                    }
                    AccountSetMemberId::AccountSet(account_set_id) => {
                        let nested_accounts =
                            Box::pin(self.find_all_accounts_by_parent_set_id(account_set_id))
                                .await?;
                        accounts.extend(nested_accounts);
                    }
                }
            }
        }
        Ok(accounts)
    }

    async fn create_equity_entry(
        &self,
        op: &mut LedgerOperation<'_>,
        accounting_period: &AccountingPeriod,
        net_earnings: Decimal,
    ) -> Result<ClosingAccountEntry, AccountingPeriodLedgerError> {
        let account = if net_earnings >= Decimal::ZERO {
            self.create_account_in_op(
                op,
                &format!("Retained Earnings {accounting_period}"),
                DebitOrCredit::Credit,
                accounting_period
                    .account_set_ids
                    .equity_retained_earnings_account_set_id,
            )
            .await?
        } else {
            self.create_account_in_op(
                op,
                &format!("Retained Losses {accounting_period}"),
                DebitOrCredit::Debit,
                accounting_period
                    .account_set_ids
                    .equity_retained_losses_account_set_id,
            )
            .await?
        };

        Ok(ClosingAccountEntry {
            account_id: account.id.into(),
            currency: Currency::USD,
            amount: net_earnings.abs(),
            direction: account.values().normal_balance_type,
        })
    }

    async fn create_account_in_op(
        &self,
        op: &mut LedgerOperation<'_>,
        name: &str,
        normal_balance_type: DebitOrCredit,
        parent_account_set: AccountSetId,
    ) -> Result<Account, AccountingPeriodLedgerError> {
        let id = AccountId::new();
        let new_ledger_account = NewAccount::builder()
            .id(id)
            .name(name)
            .code(id.to_string())
            .normal_balance_type(normal_balance_type)
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

        Ok(ledger_account)
    }

    pub const CHART_OF_ACCOUNTS_INTEGRATION_KEY: &'static str = "chart_of_accounts_integration";

    pub async fn get_chart_of_accounts_integration_config(
        &self,
        root_chart_account_set_id: AccountSetId,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, AccountingPeriodLedgerError> {
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
    ) -> Result<(), AccountingPeriodLedgerError> {
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
