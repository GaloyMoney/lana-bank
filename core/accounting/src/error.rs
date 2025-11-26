use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CoreAccountingError {
    #[error("CoreAccountingError - ChartOfAccounts: {0}")]
    ChartOfAccountsError(#[from] super::chart_of_accounts_error::ChartOfAccountsError),
    #[error("CoreAccountingError - LedgerAccount: {0}")]
    LedgerAccountError(#[from] super::ledger_account::error::LedgerAccountError),
    #[error("CoreAccountingError - ManualTransaction: {0}")]
    ManualTransactionError(#[from] super::manual_transaction::error::ManualTransactionError),
    #[error("CoreAccountingError - LedgerTransaction: {0}")]
    LedgerTransactionError(#[from] super::ledger_transaction::error::LedgerTransactionError),
    #[error("CoreAccountingError - AccountCodeParseError: {0}")]
    AccountCodeParseError(#[from] super::AccountCodeParseError),
    #[error("CoreAccountingError - AccountingCsvExportError: {0}")]
    AccountingCsvExportError(#[from] super::csv::error::AccountingCsvExportError),
    #[error("CoreAccountingError - TrialBalanceError: {0}")]
    TrialBalance(#[from] super::trial_balance::error::TrialBalanceError),
    #[error("CoreAccountingError - FiscalYearError: {0}")]
    FiscalYearError(#[from] super::fiscal_year::error::FiscalYearError),
}

impl ErrorSeverity for CoreAccountingError {
    fn severity(&self) -> Level {
        match self {
            // Most accounting errors are system-level issues
            Self::ChartOfAccountsError(e) => e.severity(),
            Self::LedgerAccountError(e) => e.severity(),
            Self::ManualTransactionError(e) => e.severity(),
            Self::LedgerTransactionError(e) => e.severity(),
            Self::AccountCodeParseError(e) => e.severity(),
            Self::AccountingCsvExportError(e) => e.severity(),
            Self::TrialBalance(e) => e.severity(),
            Self::FiscalYearError(e) => e.severity(),
        }
    }
}
