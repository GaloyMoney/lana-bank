use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(thiserror::Error, Debug)]
pub enum CollateralError {
    #[error("CollateralError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CollateralError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("CollateralError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("CollateralError - CollateralLedgerError: {0}")]
    CollateralLedgerError(#[from] super::ledger::CollateralLedgerError),
    #[error("CollateralError - ManualUpdateError: Cannot update collateral with a custodian")]
    ManualUpdateError,
    #[error("CollateralError - NoActiveLiquidation")]
    NoActiveLiquidation,
    #[error("CollateralError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("CollateralError - LiquidationError: {0}")]
    LiquidationError(#[from] super::liquidation::LiquidationError),
    #[error("CollateralError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CollateralError - LedgerTransactionInitiatorParseError: {0}")]
    LedgerTransactionInitiatorParseError(
        #[from] core_accounting::LedgerTransactionInitiatorParseError,
    ),
}

impl ErrorSeverity for CollateralError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::CollateralLedgerError(e) => e.severity(),
            Self::ManualUpdateError => Level::WARN,
            Self::NoActiveLiquidation => Level::WARN,
            Self::JobError(_) => Level::ERROR,
            Self::LiquidationError(e) => e.severity(),
            Self::AuthorizationError(e) => e.severity(),
            Self::LedgerTransactionInitiatorParseError(e) => e.severity(),
        }
    }
}

es_entity::from_es_entity_error!(CollateralError);
