use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreChartOfAccountError {
    #[error("CoreChartOfAccountError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CoreChartOfAccountError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CoreChartOfAccountError - ChartOfAccountError: {0}")]
    ChartOfAccountError(#[from] crate::chart_of_accounts::error::ChartOfAccountError),
    #[error("CoreChartOfAccountError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("ChartOfAccountLedgerError - CalaAccountError: {0}")]
    CalaAccount(#[from] cala_ledger::account::error::AccountError),
}
