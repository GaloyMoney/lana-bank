use thiserror::Error;
use cala_ledger::error::LedgerError;
use tracing::Level;
use tracing_utils::error_severity::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CollectionsLedgerError {
    #[error("CollectionsLedgerError - Ledger: {0}")]
    Ledger(#[from] LedgerError),
    #[error("CollectionsLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl ErrorSeverity for CollectionsLedgerError {
    fn severity(&self) -> Level {
        match self {
            Self::Ledger(_) => Level::ERROR,
            Self::Sqlx(_) => Level::ERROR,
        }
    }
}
