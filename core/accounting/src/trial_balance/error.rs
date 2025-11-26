use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum TrialBalanceError {
    #[error("TrialBalanceError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("TrialBalanceError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("TrialBalanceError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("TrialBalanceError - TrialBalanceLedgerError: {0}")]
    TrialBalanceLedgerError(#[from] super::ledger::error::TrialBalanceLedgerError),
}

impl ErrorSeverity for TrialBalanceError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::AuditError(_) => Level::ERROR,
            Self::AuthorizationError(_) => Level::ERROR,
            Self::TrialBalanceLedgerError(e) => e.severity(),
        }
    }
}
