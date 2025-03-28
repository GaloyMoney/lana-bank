use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreAccountingError {
    #[error("CoreAccountingError - ChartOfAccounts: {0}")]
    ChartOfAccountsError(#[from] super::chart_of_accounts_error::ChartOfAccountsError),
}
