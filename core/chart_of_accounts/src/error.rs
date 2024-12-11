use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreChartOfAccountsError {
    #[error("CoreChartOfAccountsError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CoreChartOfAccountsError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CoreChartOfAccountsError - ChartOfAccountsError: {0}")]
    ChartOfAccountsError(#[from] crate::chart_of_accounts::error::ChartOfAccountsError),
    #[error("CoreChartOfAccountsError - ChartOfAccountsLedgerError: {0}")]
    ChartOfAccountsLedgerError(#[from] crate::ledger::error::ChartOfAccountsLedgerError),
}
