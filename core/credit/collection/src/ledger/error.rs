use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CollectionLedgerError {
    #[error("CollectionLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CollectionLedgerError - Ledger: {0}")]
    Ledger(Box<dyn std::error::Error + Send + Sync>),
    #[error("CollectionLedgerError - PaymentAmountGreaterThanOutstandingObligations")]
    PaymentAmountGreaterThanOutstandingObligations,
}

impl CollectionLedgerError {
    pub fn from_ledger(e: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Ledger(Box::new(e))
    }
}

impl ErrorSeverity for CollectionLedgerError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Ledger(_) => Level::ERROR,
            Self::PaymentAmountGreaterThanOutstandingObligations => Level::WARN,
        }
    }
}
