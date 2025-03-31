use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreAccountingError {
    #[error("CoreAccountingError - ChartOfAccountsNotFoundByReference: {0}")]
    ChartOfAccountsNotFoundByReference(String),
    #[error("CoreAccountingError - ChartOfAccounts: {0}")]
    ChartOfAccountsError(#[from] super::chart_of_accounts_error::ChartOfAccountsError),
    #[error("CoreAccountingError - LedgerAccount: {0}")]
    LedgerAccountError(#[from] super::ledger_account::error::LedgerAccountError),
    #[error("CoreAccountingError - AccountCodeParseError: {0}")]
    AccountCodeParseError(#[from] super::AccountCodeParseError),
}
