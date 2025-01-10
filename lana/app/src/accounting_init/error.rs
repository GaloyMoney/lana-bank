use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountingInitError {
    #[error("AccountingInitError - CoreChartOfAccountsError: {0}")]
    CoreChartOfAccountsError(#[from] chart_of_accounts::error::CoreChartOfAccountsError),
    #[error("ApplicationError - JournalError: {0}")]
    JournalError(#[from] cala_ledger::journal::error::JournalError),
}
