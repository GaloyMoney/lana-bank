use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CollateralLedgerError {
    #[error("CollateralLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CollateralLedgerError - Ledger: {0}")]
    Ledger(Box<dyn std::error::Error + Send + Sync>),
}

impl CollateralLedgerError {
    pub fn from_ledger(e: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Ledger(Box::new(e))
    }
}

impl ErrorSeverity for CollateralLedgerError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Ledger(_) => Level::ERROR,
        }
    }
}
