#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod balance_sheet;
pub mod chart_of_accounts;
pub mod csv;
pub mod error;
pub mod event;
pub mod fiscal_year;
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
use document_storage::DocumentStorage;
use job::Jobs;
use manual_transaction::ManualTransactions;
use obix::out::{Outbox, OutboxEventMarker};
use tracing::instrument;
use tracing_macros::record_error_severity;

pub use balance_sheet::{BalanceSheet, BalanceSheets};
pub use chart_of_accounts::{Chart, ChartOfAccounts, error as chart_of_accounts_error, tree};
pub use csv::AccountingCsvExports;
use error::CoreAccountingError;
pub use event::{CSV_EXPORT_EVENT_TYPE, CoreAccountingEvent};
pub use fiscal_year::{
    FiscalYear, FiscalYears, FiscalYearsByCreatedAtCursor, error as fiscal_year_error,
};
pub use journal::{Journal, error as journal_error};
pub use ledger_account::{LedgerAccount, LedgerAccountChildrenCursor, LedgerAccounts};
pub use ledger_transaction::{LedgerTransaction, LedgerTransactions};
pub use manual_transaction::ManualEntryInput;
pub use primitives::AccountSetMember;
pub use primitives::*;
pub use profit_and_loss::{ProfitAndLossStatement, ProfitAndLossStatements};
pub use transaction_templates::TransactionTemplates;
pub use trial_balance::{TrialBalanceRoot, TrialBalances};

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::chart_of_accounts::ChartEvent;
    pub use crate::chart_of_accounts::chart_node::ChartNodeEvent;
    pub use crate::fiscal_year::FiscalYearEvent;
    pub use crate::manual_transaction::ManualTransactionEvent;
}

pub struct CoreAccounting<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreAccountingEvent>,
{
    clock: ClockHandle,
    authz: Perms,
    chart_of_accounts: ChartOfAccounts<Perms>,
    journal: Journal<Perms>,
    ledger_accounts: LedgerAccounts<Perms>,
    ledger_transactions: LedgerTransactions<Perms>,
    manual_transactions: ManualTransactions<Perms>,
    profit_and_loss: ProfitAndLossStatements<Perms>,
    transaction_templates: TransactionTemplates<Perms>,
    balance_sheets: BalanceSheets<Perms>,
    csvs: AccountingCsvExports<Perms, E>,
    trial_balances: TrialBalances<Perms>,
    fiscal_year: FiscalYears<Perms>,
}

impl<Perms, E> Clone for CoreAccounting<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreAccountingEvent>,
{
    fn clone(&self) -> Self {
        Self {
            clock: self.clock.clone(),
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
            fiscal_year: self.fiscal_year.clone(),
        }
    }
}

impl<Perms, E> CoreAccounting<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
    E: OutboxEventMarker<CoreAccountingEvent>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        cala: &CalaLedger,
        journal_id: CalaJournalId,
        document_storage: DocumentStorage,
        jobs: &mut Jobs,
        outbox: &Outbox<E>,
    ) -> Self {
        let clock = jobs.clock().clone();
        let chart_of_accounts = ChartOfAccounts::new(pool, clock.clone(), authz, cala, journal_id);
        let fiscal_year = FiscalYears::new(pool, clock.clone(), authz, &chart_of_accounts);
        let journal = Journal::new(authz, cala, journal_id);
        let ledger_accounts = LedgerAccounts::new(authz, cala, journal_id);
        let manual_transactions = ManualTransactions::new(
            pool,
            authz,
            &chart_of_accounts,
            cala,
            journal_id,
            clock.clone(),
        );
        let ledger_transactions = LedgerTransactions::new(authz, cala);
        let profit_and_loss = ProfitAndLossStatements::new(pool, authz, cala, journal_id);
        let transaction_templates = TransactionTemplates::new(authz, cala);
        let balance_sheets = BalanceSheets::new(pool, authz, cala, journal_id);
        let csvs =
            AccountingCsvExports::new(authz, jobs, document_storage, &ledger_accounts, outbox);
        let trial_balances = TrialBalances::new(pool, authz, cala, journal_id);
        Self {
            clock,
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
            fiscal_year,
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

    pub fn csvs(&self) -> &AccountingCsvExports<Perms, E> {
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

    pub fn fiscal_year(&self) -> &FiscalYears<Perms> {
        &self.fiscal_year
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.find_ledger_account_by_id", skip(self))]
    pub async fn find_ledger_account_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        id: impl Into<LedgerAccountId> + std::fmt::Debug,
    ) -> Result<Option<LedgerAccount>, CoreAccountingError> {
        let chart = self.chart_of_accounts.find_by_reference(chart_ref).await?;

        Ok(self.ledger_accounts.find_by_id(sub, &chart, id).await?)
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.find_ledger_account_by_code", skip(self))]
    pub async fn find_ledger_account_by_code(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        code: String,
    ) -> Result<Option<LedgerAccount>, CoreAccountingError> {
        let chart = self.chart_of_accounts.find_by_reference(chart_ref).await?;
        Ok(self
            .ledger_accounts
            .find_by_code(sub, &chart, code.parse()?)
            .await?)
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.find_all_ledger_accounts", skip(self))]
    pub async fn find_all_ledger_accounts<T: From<LedgerAccount>>(
        &self,
        chart_ref: &str,
        ids: &[LedgerAccountId],
    ) -> Result<HashMap<LedgerAccountId, T>, CoreAccountingError> {
        let chart = self.chart_of_accounts.find_by_reference(chart_ref).await?;
        Ok(self.ledger_accounts.find_all(&chart, ids).await?)
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.list_all_account_flattened", skip(self))]
    pub async fn list_all_account_flattened(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        from: chrono::NaiveDate,
        until: Option<chrono::NaiveDate>,
    ) -> Result<Vec<LedgerAccount>, CoreAccountingError> {
        let chart = self.chart_of_accounts.find_by_reference(chart_ref).await?;

        Ok(self
            .ledger_accounts()
            .list_all_account_flattened(sub, &chart, from, until, true)
            .await?)
    }

    #[record_error_severity]
    #[instrument(
        name = "core_accounting.execute_manual_transaction",
        skip(self, entries)
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
                effective.unwrap_or_else(|| self.clock.today()),
                entries,
            )
            .await?;

        let ledger_tx_id = tx.ledger_transaction_id;
        let mut txs = self.ledger_transactions.find_all(&[ledger_tx_id]).await?;
        Ok(txs
            .remove(&ledger_tx_id)
            .expect("Could not find LedgerTransaction"))
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.import_csv_with_base_config", skip(self, data))]
    pub async fn import_csv_with_base_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        data: String,
        base_config: AccountingBaseConfig,
        balance_sheet_ref: &str,
        profit_and_loss_ref: &str,
        trial_balance_ref: &str,
    ) -> Result<Chart, CoreAccountingError> {
        let mut op = self.chart_of_accounts.begin_op().await?;

        let (chart, new_trial_balance_account_ids) = self
            .chart_of_accounts
            .import_from_csv_with_base_config_in_op(&mut op, sub, chart_ref, data, base_config)
            .await?;

        let resolved = chart
            .resolve_accounting_base_config()
            .ok_or(chart_of_accounts_error::ChartOfAccountsError::BaseConfigNotInitialized)?;

        self.trial_balances
            .add_new_chart_accounts_to_trial_balance_in_op(
                &mut op,
                trial_balance_ref,
                &new_trial_balance_account_ids,
            )
            .await?;

        self.balance_sheets
            .link_chart_account_sets_in_op(&mut op, balance_sheet_ref.to_string(), &resolved)
            .await?;

        self.profit_and_loss
            .link_chart_account_sets_in_op(&mut op, profit_and_loss_ref.to_string(), &resolved)
            .await?;

        op.commit().await?;

        Ok(chart)
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.import_csv", skip(self))]
    pub async fn import_csv(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        data: String,
        trial_balance_ref: &str,
    ) -> Result<Chart, CoreAccountingError> {
        let (chart, new_account_set_ids) = self
            .chart_of_accounts()
            .import_from_csv(sub, chart_ref, data)
            .await?;
        if let Some(new_account_set_ids) = new_account_set_ids {
            self.trial_balances()
                .add_new_chart_accounts_to_trial_balance(trial_balance_ref, &new_account_set_ids)
                .await?;
        }
        Ok(chart)
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.init_fiscal_year_for_chart", skip(self))]
    pub async fn init_fiscal_year_for_chart(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        opened_as_of: impl Into<chrono::NaiveDate> + std::fmt::Debug,
    ) -> Result<FiscalYear, CoreAccountingError> {
        let chart = self
            .chart_of_accounts()
            .find_by_reference(chart_ref)
            .await?;

        Ok(self
            .fiscal_year()
            .init_for_chart(sub, opened_as_of, chart.id)
            .await?)
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.list_fiscal_years_for_chart", skip(self))]
    pub async fn list_fiscal_years_for_chart(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        query: es_entity::PaginatedQueryArgs<FiscalYearsByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<FiscalYear, FiscalYearsByCreatedAtCursor>,
        CoreAccountingError,
    > {
        let chart = self
            .chart_of_accounts()
            .find_by_reference(chart_ref)
            .await?;
        Ok(self
            .fiscal_year()
            .list_for_chart_id(sub, chart.id, query)
            .await?)
    }

    #[record_error_severity]
    #[instrument(
        name = "core_accounting.fiscal_year.find_for_chart_by_year",
        skip(self)
    )]
    pub async fn find_fiscal_year_for_chart_by_year(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        year: &str,
    ) -> Result<Option<FiscalYear>, CoreAccountingError> {
        let chart = self
            .chart_of_accounts()
            .find_by_reference(chart_ref)
            .await?;

        Ok(self
            .fiscal_year()
            .find_by_chart_id_and_year(sub, chart.id, year)
            .await?)
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.add_root_node", skip(self))]
    pub async fn add_root_node(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        spec: AccountSpec,
        trial_balance_ref: &str,
    ) -> Result<Chart, CoreAccountingError> {
        let (chart, new_account_set_id) = self
            .chart_of_accounts()
            .add_root_node(sub, chart_ref, spec)
            .await?;
        if let Some(new_account_set_id) = new_account_set_id {
            self.trial_balances()
                .add_new_chart_accounts_to_trial_balance(trial_balance_ref, &[new_account_set_id])
                .await?;
        }

        Ok(chart)
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.add_child_node", skip(self))]
    pub async fn add_child_node(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        parent_code: AccountCode,
        code: AccountCode,
        name: AccountName,
        trial_balance_ref: &str,
    ) -> Result<Chart, CoreAccountingError> {
        let (chart, new_account_set_id) = self
            .chart_of_accounts()
            .add_child_node(sub, chart_ref, parent_code, code, name)
            .await?;
        if let Some(new_account_set_id) = new_account_set_id {
            self.trial_balances()
                .add_new_chart_accounts_to_trial_balance(trial_balance_ref, &[new_account_set_id])
                .await?;
        }

        Ok(chart)
    }
}
