use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum LedgerAccountError {
    #[error("LedgerAccountError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("LedgerAccountError - LedgerAccountLedgerError: {0}")]
    LedgerAccountLedgerError(#[from] super::ledger::error::LedgerAccountLedgerError),
}

impl ErrorSeverity for LedgerAccountError {
    fn severity(&self) -> Level {
        match self {
            Self::AuthorizationError(_) => Level::ERROR,
            Self::LedgerAccountLedgerError(e) => e.severity(),
        }
    }
}
