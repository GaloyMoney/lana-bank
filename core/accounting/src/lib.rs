#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

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
pub mod transaction_templates;
pub mod trial_balance;

use std::collections::HashMap;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use cloud_storage::Storage;
use job::Jobs;
use manual_transaction::ManualTransactions;
use tracing::instrument;

pub use balance_sheet::{BalanceSheet, BalanceSheets};
pub use chart_of_accounts::{
    Chart, ChartEvent, ChartOfAccounts, error as chart_of_accounts_error, tree,
};
pub use csv::{AccountingCsvEvent, AccountingCsvs};
use error::CoreAccountingError;
pub use journal::{Journal, error as journal_error};
pub use ledger_account::{LedgerAccount, LedgerAccountChildrenCursor, LedgerAccounts};
pub use ledger_transaction::{LedgerTransaction, LedgerTransactions};
pub use manual_transaction::{ManualEntryInput, ManualTransactionEvent};
pub use primitives::*;
pub use profit_and_loss::{ProfitAndLossStatement, ProfitAndLossStatements};
pub use transaction_templates::TransactionTemplates;
pub use trial_balance::{TrialBalanceRoot, TrialBalances};

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
    csvs: AccountingCsvs<Perms>,
    trial_balances: TrialBalances<Perms>,
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
        storage: &Storage,
        jobs: &Jobs,
    ) -> Self {
        let chart_of_accounts = ChartOfAccounts::new(pool, authz, cala, journal_id);
        let journal = Journal::new(authz, cala, journal_id);
        let ledger_accounts = LedgerAccounts::new(authz, cala, journal_id);
        let manual_transactions = ManualTransactions::new(pool, authz, cala, journal_id);
        let ledger_transactions = LedgerTransactions::new(authz, cala);
        let profit_and_loss = ProfitAndLossStatements::new(pool, authz, cala, journal_id);
        let transaction_templates = TransactionTemplates::new(authz, cala);
        let balance_sheets = BalanceSheets::new(pool, authz, cala, journal_id);
        let csvs = AccountingCsvs::new(pool, authz, jobs, storage, &ledger_accounts);
        let trial_balances = TrialBalances::new(pool, authz, cala, journal_id);
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

    pub fn csvs(&self) -> &AccountingCsvs<Perms> {
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

    #[instrument(name = "core_accounting.find_ledger_account_by_code", skip(self), err)]
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

    #[instrument(name = "core_accounting.list_account_children", skip(self), err)]
    pub async fn list_account_children(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        id: cala_ledger::AccountSetId,
        args: es_entity::PaginatedQueryArgs<LedgerAccountChildrenCursor>,
        from: chrono::NaiveDate,
        until: Option<chrono::NaiveDate>,
    ) -> Result<
        es_entity::PaginatedQueryRet<LedgerAccount, LedgerAccountChildrenCursor>,
        CoreAccountingError,
    > {
        let chart = self
            .chart_of_accounts
            .find_by_reference(chart_ref)
            .await?
            .ok_or_else(move || {
                CoreAccountingError::ChartOfAccountsNotFoundByReference(chart_ref.to_string())
            })?;

        Ok(self
            .ledger_accounts()
            .list_account_children(sub, &chart, id, args, from, until, true)
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
        let chart = self
            .chart_of_accounts
            .find_by_reference(chart_ref)
            .await?
            .ok_or_else(move || {
                CoreAccountingError::ChartOfAccountsNotFoundByReference(chart_ref.to_string())
            })?;

        let tx = self
            .manual_transactions
            .execute(
                sub,
                &chart,
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
    ) -> Result<bool, CoreAccountingError> {
        if let Some(new_account_set_ids) = self
            .chart_of_accounts()
            .import_from_csv(sub, chart_id, data)
            .await?
        {
            self.trial_balances()
                .add_new_chart_accounts_to_trial_balance(trial_balance_ref, new_account_set_ids)
                .await?;
        }

        Ok(true)
    }
}
