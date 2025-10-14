#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod annual_closing_transaction;
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
mod accounting_period;

use std::collections::HashMap;

pub use annual_closing_transaction::AnnualClosingTransactions;
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
    authz: Perms,
    chart_of_accounts: ChartOfAccounts<Perms>,
    journal: Journal<Perms>,
    ledger_accounts: LedgerAccounts<Perms>,
    ledger_transactions: LedgerTransactions<Perms>,
    manual_transactions: ManualTransactions<Perms>,
    profit_and_loss: ProfitAndLossStatements<Perms>,
    transaction_templates: TransactionTemplates<Perms>,
    balance_sheets: BalanceSheets<Perms>,
    csvs: AccountingCsvExports<Perms>,
    trial_balances: TrialBalances<Perms>,
    annual_closing_transactions: AnnualClosingTransactions<Perms>,
}

impl<Perms> Clone for CoreAccounting<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            chart_of_accounts: self.chart_of_accounts.clone(),
            journal: self.journal.clone(),
            ledger_accounts: self.ledger_accounts.clone(),
            manual_transactions: self.manual_transactions.clone(),
            ledger_transactions: self.ledger_transactions.clone(),
            profit_and_loss: self.profit_and_loss.clone(),
            transaction_templates: self.transaction_templates.clone(),
            balance_sheets: self.balance_sheets.clone(),
            csvs: self.csvs.clone(),
            trial_balances: self.trial_balances.clone(),
            annual_closing_transactions: self.annual_closing_transactions.clone(),
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
        let annual_closing_transactions =
            AnnualClosingTransactions::new(pool, authz, &chart_of_accounts, cala, journal_id);
        Self {
            authz: authz.clone(),
            chart_of_accounts,
            journal,
            ledger_accounts,
            ledger_transactions,
            manual_transactions,
            profit_and_loss,
            transaction_templates,
            balance_sheets,
            csvs,
            trial_balances,
            annual_closing_transactions,
        }
    }

    pub fn chart_of_accounts(&self) -> &ChartOfAccounts<Perms> {
        &self.chart_of_accounts
    }

    pub fn journal(&self) -> &Journal<Perms> {
        &self.journal
    }

    pub fn ledger_accounts(&self) -> &LedgerAccounts<Perms> {
        &self.ledger_accounts
    }

    pub fn ledger_transactions(&self) -> &LedgerTransactions<Perms> {
        &self.ledger_transactions
    }

    pub fn manual_transactions(&self) -> &ManualTransactions<Perms> {
        &self.manual_transactions
    }

    pub fn profit_and_loss(&self) -> &ProfitAndLossStatements<Perms> {
        &self.profit_and_loss
    }

    pub fn csvs(&self) -> &AccountingCsvExports<Perms> {
        &self.csvs
    }

    pub fn transaction_templates(&self) -> &TransactionTemplates<Perms> {
        &self.transaction_templates
    }

    pub fn balance_sheets(&self) -> &BalanceSheets<Perms> {
        &self.balance_sheets
    }

    pub fn trial_balances(&self) -> &TrialBalances<Perms> {
        &self.trial_balances
    }

    pub fn annual_closing_transactions(&self) -> &AnnualClosingTransactions<Perms> {
        &self.annual_closing_transactions
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

    #[instrument(name = "core_accounting.close_monthly", skip(self), err)]
    pub async fn close_monthly(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: ChartId,
    ) -> Result<Chart, CoreAccountingError> {
        Ok(self
            .chart_of_accounts()
            .close_monthly(sub, chart_id)
            .await?)
    }

    #[instrument(
        name = "core_accounting.execute_annual_closing_transaction",
        skip(self),
        err
    )]
    pub async fn execute_annual_closing_transaction(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        // TODO: Need both? Can lookup one from the other?
        chart_id: ChartId,
    ) -> Result<LedgerTransaction, CoreAccountingError> {
        let annual_closing_tx = self
            .annual_closing_transactions()
            .execute(
                sub,
                chart_id,
                // TODO: Where to source `reference`?
                None,
                // TODO: Add optional description to API?
                "Annual Closing".to_string(),
            )
            .await?;

        let ledger_tx_id = annual_closing_tx.ledger_transaction_id;
        Ok(self
            .ledger_transactions
            .find_by_id(sub, ledger_tx_id)
            .await?
            .ok_or_else(move || {
                CoreAccountingError::AnnualClosingTransactionNotFoundById(ledger_tx_id.to_string())
            })?)
    }

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
