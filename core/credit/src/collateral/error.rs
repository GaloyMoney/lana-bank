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
    #[error("CollateralError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
}

impl ErrorSeverity for CollateralError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::CollateralLedgerError(e) => e.severity(),
            Self::ManualUpdateError => Level::WARN,
            Self::JobError(_) => Level::ERROR,
        }
    }
}

es_entity::from_es_entity_error!(CollateralError);
