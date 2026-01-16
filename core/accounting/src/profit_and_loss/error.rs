use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum ProfitAndLossStatementError {
    #[error("ProfitAndLossStatementError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ProfitAndLossStatementError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("ProfitAndLossStatementError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("ProfitAndLossStatementError - ProfitAndLossStatementLedgerError: {0}")]
    ProfitAndLossStatementLedgerError(
        #[from] super::ledger::error::ProfitAndLossStatementLedgerError,
    ),
    #[error("ProfitAndLossStatementError - ChartOfAccountsError: {0}")]
    ChartOfAccountsError(#[from] crate::chart_of_accounts::error::ChartOfAccountsError),
    #[error("ProfitAndLossStatementError - ChartIdMismatch")]
    ChartIdMismatch,
    #[error("ProfitAndLossStatementError - AccountingBaseConfigNotFound")]
    AccountingBaseConfigNotFound,
}

impl ErrorSeverity for ProfitAndLossStatementError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::AuditError(e) => e.severity(),
            Self::AuthorizationError(e) => e.severity(),
            Self::ProfitAndLossStatementLedgerError(e) => e.severity(),
            Self::ChartOfAccountsError(e) => e.severity(),
            Self::ChartIdMismatch => Level::ERROR,
            Self::AccountingBaseConfigNotFound => Level::ERROR,
        }
    }
}
