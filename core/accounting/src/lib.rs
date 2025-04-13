#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod chart_of_accounts;
pub mod error;
pub mod journal;
pub mod ledger_account;
pub mod ledger_transaction;
pub mod manual_transaction;
mod primitives;
pub mod profit_and_loss;
pub mod transaction_templates;

use std::collections::HashMap;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use manual_transaction::ManualTransactions;
use tracing::instrument;

pub use chart_of_accounts::{Chart, ChartOfAccounts, error as chart_of_accounts_error, tree};
use error::CoreAccountingError;
pub use journal::{Journal, error as journal_error};
pub use ledger_account::{LedgerAccount, LedgerAccounts};
pub use ledger_transaction::{LedgerTransaction, LedgerTransactions};
pub use manual_transaction::ManualEntryInput;
pub use primitives::*;
pub use profit_and_loss::{ProfitAndLossStatement, ProfitAndLossStatements};
pub use transaction_templates::TransactionTemplates;

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
    ) -> Self {
        let chart_of_accounts = ChartOfAccounts::new(pool, authz, cala, journal_id);
        let journal = Journal::new(authz, cala, journal_id);
        let ledger_accounts = LedgerAccounts::new(authz, cala, journal_id);
        let manual_transactions = ManualTransactions::new(pool, authz, cala, journal_id);
        let ledger_transactions = LedgerTransactions::new(authz, cala);
        let profit_and_loss = ProfitAndLossStatements::new(pool, authz, cala, journal_id);
        let transaction_templates = TransactionTemplates::new(authz, cala);
        Self {
            authz: authz.clone(),
            chart_of_accounts,
            journal,
            ledger_accounts,
            ledger_transactions,
            manual_transactions,
            profit_and_loss,
            transaction_templates,
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

    pub fn transaction_templates(&self) -> &TransactionTemplates<Perms> {
        &self.transaction_templates
    }

    #[instrument(name = "core_accounting.find_ledger_account_by_code", skip(self))]
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

    #[instrument(name = "core_accounting.find_ledger_account_by_code", skip(self))]
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

    #[instrument(name = "core_accounting.find_all_ledger_accounts", skip(self))]
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
}
