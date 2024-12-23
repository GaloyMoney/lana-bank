use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountingInitError {
    #[error("AccountingInitError - CoreChartOfAccountError: {0}")]
    CoreChartOfAccountError(#[from] chart_of_accounts::error::CoreChartOfAccountError),
    #[error("ApplicationError - JournalError: {0}")]
    JournalError(#[from] cala_ledger::journal::error::JournalError),
}
