use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreAccountingError {
    #[error("CoreAccountingError - ChartOfAccountsNotFoundByReference: {0}")]
    ChartOfAccountsNotFoundByReference(String),
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
}
