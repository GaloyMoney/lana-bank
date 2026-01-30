use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum LiquidationError {
    #[error("LiquidationError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("LiquidationError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("LiquidationError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("LiquidationError - Ledger: {0}")]
    LedgerError(#[from] super::ledger::LiquidationLedgerError),
    #[error("LiquidationError - AlreadySatifissed")]
    AlreadySatisfied,
    #[error("LiquidationError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CoreCreditError - LedgerTransactionInitiatorParseError: {0}")]
    LedgerTransactionInitiatorParseError(
        #[from] core_accounting::LedgerTransactionInitiatorParseError,
    ),
    #[error("LiquidationError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("LiquidationError - CollateralError: {0}")]
    CollateralError(Box<crate::collateral::error::CollateralError>),
}

es_entity::from_es_entity_error!(LiquidationError);

impl From<crate::collateral::error::CollateralError> for LiquidationError {
    fn from(err: crate::collateral::error::CollateralError) -> Self {
        LiquidationError::CollateralError(Box::new(err))
    }
}

impl ErrorSeverity for LiquidationError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::LedgerError(e) => e.severity(),
            Self::AlreadySatisfied => Level::WARN,
            Self::AuthorizationError(e) => e.severity(),
            Self::LedgerTransactionInitiatorParseError(e) => e.severity(),
            Self::JobError(_) => Level::ERROR,
            Self::CollateralError(e) => e.severity(),
        }
    }
}
