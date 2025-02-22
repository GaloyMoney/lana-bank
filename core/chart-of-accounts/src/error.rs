use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreChartOfAccountsError {
    #[error("CoreChartOfAccountsError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CoreChartOfAccountsError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CoreChartOfAccountsError - ChartError: {0}")]
    ChartError(#[from] crate::chart_of_accounts::error::ChartError),
    #[error("CoreChartOfAccountsError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("CoreChartOfAccountsError - CalaLedgerError: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("CoreChartOfAccountsError - CalaAccountError: {0}")]
    CalaAccount(#[from] cala_ledger::account::error::AccountError),
    #[error("CoreChartOfAccountsError - CalaAccountSetError: {0}")]
    CalaAccountSet(#[from] cala_ledger::account_set::error::AccountSetError),
}
