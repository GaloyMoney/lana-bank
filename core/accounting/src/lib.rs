#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod accounting_period;
pub mod balance_sheet;
pub mod chart_of_accounts;
pub mod csv;
pub mod error;
pub mod journal;
pub mod ledger_account;
pub mod ledger_transaction;
pub mod manual_transaction;
mod primitives;
pub mod profit_and_loss;
mod time;
pub mod transaction_templates;
pub mod trial_balance;

use std::{collections::HashMap, sync::Arc};

use audit::AuditSvc;
use authz::PermissionCheck;
pub use balance_sheet::{BalanceSheet, BalanceSheets};
use cala_ledger::CalaLedger;
pub use chart_of_accounts::{
    Chart, ChartOfAccounts, PeriodClosing, error as chart_of_accounts_error, tree,
};
pub use csv::AccountingCsvExports;
use document_storage::DocumentStorage;
use error::CoreAccountingError;
use job::Jobs;
pub use journal::{Journal, error as journal_error};
pub use ledger_account::{LedgerAccount, LedgerAccountChildrenCursor, LedgerAccounts};
pub use ledger_transaction::{LedgerTransaction, LedgerTransactions};
pub use manual_transaction::ManualEntryInput;
use manual_transaction::ManualTransactions;
pub use primitives::*;
pub use profit_and_loss::{ProfitAndLossStatement, ProfitAndLossStatements};
use tracing::instrument;
pub use transaction_templates::TransactionTemplates;
pub use trial_balance::{TrialBalanceRoot, TrialBalances};
pub use accounting_period::AccountingPeriods;

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::chart_of_accounts::ChartEvent;
    pub use crate::chart_of_accounts::chart_node::ChartNodeEvent;
    pub use crate::manual_transaction::ManualTransactionEvent;
}

pub struct CoreAccounting<Perms>
where
    Perms: PermissionCheck,
{
    authz: Arc<Perms>,
    chart_of_accounts: Arc<ChartOfAccounts<Perms>>,
    journal: Arc<Journal<Perms>>,
    ledger_accounts: Arc<LedgerAccounts<Perms>>,
    ledger_transactions: Arc<LedgerTransactions<Perms>>,
    manual_transactions: Arc<ManualTransactions<Perms>>,
    profit_and_loss: Arc<ProfitAndLossStatements<Perms>>,
    transaction_templates: Arc<TransactionTemplates<Perms>>,
    balance_sheets: Arc<BalanceSheets<Perms>>,
    csvs: Arc<AccountingCsvExports<Perms>>,
    trial_balances: Arc<TrialBalances<Perms>>,
    accounting_periods: Arc<AccountingPeriods<Perms>>,
}

impl<Perms> Clone for CoreAccounting<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            authz: Arc::clone(&self.authz),
            chart_of_accounts: Arc::clone(&self.chart_of_accounts),
            journal: Arc::clone(&self.journal),
            ledger_accounts: Arc::clone(&self.ledger_accounts),
            manual_transactions: Arc::clone(&self.manual_transactions),
            ledger_transactions: Arc::clone(&self.ledger_transactions),
            profit_and_loss: Arc::clone(&self.profit_and_loss),
            transaction_templates: Arc::clone(&self.transaction_templates),
            balance_sheets: Arc::clone(&self.balance_sheets),
            csvs: Arc::clone(&self.csvs),
            trial_balances: Arc::clone(&self.trial_balances),
            accounting_periods: Arc::clone(&self.accounting_periods),
        }
    }
}

impl<Perms> CoreAccounting<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        cala: &CalaLedger,
        journal_id: CalaJournalId,
        document_storage: DocumentStorage,
        jobs: &Jobs,
    ) -> Self {
        let chart_of_accounts = ChartOfAccounts::new(pool, authz, cala, journal_id);
        let journal = Journal::new(authz, cala, journal_id);
        let ledger_accounts = LedgerAccounts::new(authz, cala, journal_id);
        let manual_transactions =
            ManualTransactions::new(pool, authz, &chart_of_accounts, cala, journal_id);
        let ledger_transactions = LedgerTransactions::new(authz, cala);
        let profit_and_loss = ProfitAndLossStatements::new(pool, authz, cala, journal_id);
        let transaction_templates = TransactionTemplates::new(authz, cala);
        let balance_sheets = BalanceSheets::new(pool, authz, cala, journal_id);
        let csvs = AccountingCsvExports::new(authz, jobs, document_storage, &ledger_accounts);
        let trial_balances = TrialBalances::new(pool, authz, cala, journal_id);
        let accounting_periods =
            AccountingPeriods::new(authz, pool, cala, journal_id);
        Self {
            authz: Arc::new(authz.clone()),
            chart_of_accounts: Arc::new(chart_of_accounts),
            journal: Arc::new(journal),
            ledger_accounts: Arc::new(ledger_accounts),
            ledger_transactions: Arc::new(ledger_transactions),
            manual_transactions: Arc::new(manual_transactions),
            profit_and_loss: Arc::new(profit_and_loss),
            transaction_templates: Arc::new(transaction_templates),
            balance_sheets: Arc::new(balance_sheets),
            csvs: Arc::new(csvs),
            trial_balances: Arc::new(trial_balances),
            accounting_periods: Arc::new(accounting_periods),
        }
    }

    pub fn chart_of_accounts(&self) -> &ChartOfAccounts<Perms> {
        &*self.chart_of_accounts
    }

    pub fn journal(&self) -> &Journal<Perms> {
        &*self.journal
    }

    pub fn ledger_accounts(&self) -> &LedgerAccounts<Perms> {
        &*self.ledger_accounts
    }

    pub fn ledger_transactions(&self) -> &LedgerTransactions<Perms> {
        &*self.ledger_transactions
    }

    pub fn manual_transactions(&self) -> &ManualTransactions<Perms> {
        &*self.manual_transactions
    }

    pub fn profit_and_loss(&self) -> &ProfitAndLossStatements<Perms> {
        &*self.profit_and_loss
    }

    pub fn csvs(&self) -> &AccountingCsvExports<Perms> {
        &*self.csvs
    }

    pub fn transaction_templates(&self) -> &TransactionTemplates<Perms> {
        &*self.transaction_templates
    }

    pub fn balance_sheets(&self) -> &BalanceSheets<Perms> {
        &*self.balance_sheets
    }

    pub fn trial_balances(&self) -> &TrialBalances<Perms> {
        &*self.trial_balances
    }

    pub fn accounting_periods(&self) -> &AccountingPeriods<Perms> {
        &self.accounting_periods
    }

    #[instrument(name = "core_accounting.find_ledger_account_by_id", skip(self), err)]
    pub async fn find_ledger_account_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        id: impl Into<LedgerAccountId> + std::fmt::Debug,
    ) -> Result<Option<LedgerAccount>, CoreAccountingError> {
        let chart = self
            .chart_of_accounts
            .find_by_reference(chart_ref)
            .await?
            .ok_or_else(move || {
                CoreAccountingError::ChartOfAccountsNotFoundByReference(chart_ref.to_string())
            })?;

        Ok(self.ledger_accounts.find_by_id(sub, &chart, id).await?)
    }

    #[instrument(name = "core_accounting.find_ledger_account_by_code", skip(self), err)]
    pub async fn find_ledger_account_by_code(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        code: String,
    ) -> Result<Option<LedgerAccount>, CoreAccountingError> {
        let chart = self
            .chart_of_accounts
            .find_by_reference(chart_ref)
            .await?
            .ok_or_else(move || {
                CoreAccountingError::ChartOfAccountsNotFoundByReference(chart_ref.to_string())
            })?;
        Ok(self
            .ledger_accounts
            .find_by_code(sub, &chart, code.parse()?)
            .await?)
    }

    #[instrument(name = "core_accounting.find_all_ledger_accounts", skip(self), err)]
    pub async fn find_all_ledger_accounts<T: From<LedgerAccount>>(
        &self,
        chart_ref: &str,
        ids: &[LedgerAccountId],
    ) -> Result<HashMap<LedgerAccountId, T>, CoreAccountingError> {
        let chart = self
            .chart_of_accounts
            .find_by_reference(chart_ref)
            .await?
            .ok_or_else(move || {
                CoreAccountingError::ChartOfAccountsNotFoundByReference(chart_ref.to_string())
            })?;
        Ok(self.ledger_accounts.find_all(&chart, ids).await?)
    }

    #[instrument(name = "core_accounting.list_all_account_children", skip(self), err)]
    pub async fn list_all_account_children(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        id: cala_ledger::AccountSetId,
        from: chrono::NaiveDate,
        until: Option<chrono::NaiveDate>,
    ) -> Result<Vec<LedgerAccount>, CoreAccountingError> {
        let chart = self
            .chart_of_accounts
            .find_by_reference(chart_ref)
            .await?
            .ok_or_else(move || {
                CoreAccountingError::ChartOfAccountsNotFoundByReference(chart_ref.to_string())
            })?;

        Ok(self
            .ledger_accounts()
            .list_all_account_children(sub, &chart, id, from, until, true)
            .await?)
    }

    #[instrument(
        name = "core_accounting.execute_manual_transaction",
        skip(self, entries),
        err
    )]
    pub async fn execute_manual_transaction(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        reference: Option<String>,
        description: String,
        effective: Option<chrono::NaiveDate>,
        entries: Vec<ManualEntryInput>,
    ) -> Result<LedgerTransaction, CoreAccountingError> {
        let tx = self
            .manual_transactions
            .execute(
                sub,
                chart_ref,
                reference,
                description,
                effective.unwrap_or_else(|| chrono::Utc::now().date_naive()),
                entries,
            )
            .await?;

        let ledger_tx_id = tx.ledger_transaction_id;
        let mut txs = self.ledger_transactions.find_all(&[ledger_tx_id]).await?;
        Ok(txs
            .remove(&ledger_tx_id)
            .expect("Could not find LedgerTransaction"))
    }

    #[instrument(name = "core_accounting.import_csv", skip(self), err)]
    pub async fn import_csv(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: ChartId,
        data: String,
        trial_balance_ref: &str,
    ) -> Result<Chart, CoreAccountingError> {
        let (chart, new_account_set_ids) = self
            .chart_of_accounts()
            .import_from_csv(sub, chart_id, data)
            .await?;
        if let Some(new_account_set_ids) = new_account_set_ids {
            self.trial_balances()
                .add_new_chart_accounts_to_trial_balance(trial_balance_ref, &new_account_set_ids)
                .await?;
        }

        Ok(chart)
    }

    // #[instrument(name = "core_accounting.close_monthly", skip(self), err)]
    // pub async fn close_monthly(
    //     &self,
    //     sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    //     chart_id: ChartId,
    // ) -> Result<Chart, CoreAccountingError> {
    //     Ok(self
    //         .accounting_periods()
    //         .close_month(sub, chart_id)
    //         .await?)
    // }

    // #[instrument(
    //     name = "core_accounting.execute_annual_closing_transaction",
    //     skip(self),
    //     err
    // )]
    // pub async fn execute_annual_closing_transaction(
    //     &self,
    //     sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    //     // TODO: Need both? Can lookup one from the other?
    //     chart_id: ChartId,
    // ) -> Result<LedgerTransaction, CoreAccountingError> {
    //     let annual_closing_tx = self
    //         .annual_closing_transactions()
    //         .execute(
    //             sub,
    //             chart_id,
    //             // TODO: Where to source `reference`?
    //             None,
    //             // TODO: Add optional description to API?
    //             "Annual Closing".to_string(),
    //         )
    //         .await?;

    //     let ledger_tx_id = annual_closing_tx.ledger_transaction_id;
    //     Ok(self
    //         .ledger_transactions
    //         .find_by_id(sub, ledger_tx_id)
    //         .await?
    //         .ok_or_else(move || {
    //             CoreAccountingError::AnnualClosingTransactionNotFoundById(ledger_tx_id.to_string())
    //         })?)
    // }

    #[instrument(name = "core_accounting.add_root_node", skip(self), err)]
    pub async fn add_root_node(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: ChartId,
        spec: AccountSpec,
        trial_balance_ref: &str,
    ) -> Result<Chart, CoreAccountingError> {
        let (chart, new_account_set_id) = self
            .chart_of_accounts()
            .add_root_node(sub, chart_id, spec)
            .await?;
        if let Some(new_account_set_id) = new_account_set_id {
            self.trial_balances()
                .add_new_chart_accounts_to_trial_balance(trial_balance_ref, &[new_account_set_id])
                .await?;
        }

        Ok(chart)
    }

    #[instrument(name = "core_accounting.add_child_node", skip(self), err)]
    pub async fn add_child_node(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: ChartId,
        parent_code: AccountCode,
        code: AccountCode,
        name: AccountName,
        trial_balance_ref: &str,
    ) -> Result<Chart, CoreAccountingError> {
        let (chart, new_account_set_id) = self
            .chart_of_accounts()
            .add_child_node(sub, chart_id, parent_code, code, name)
            .await?;
        if let Some(new_account_set_id) = new_account_set_id {
            self.trial_balances()
                .add_new_chart_accounts_to_trial_balance(trial_balance_ref, &[new_account_set_id])
                .await?;
        }

        Ok(chart)
    }
}
