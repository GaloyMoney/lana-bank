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
    #[error("CollateralError - CreditError: {0}")]
    CreditLedgerError(#[from] crate::ledger::error::CreditLedgerError),
    #[error("CollateralError - ManualUpdateError: Cannot update collateral with a custodian")]
    ManualUpdateError,
    #[error("CollateralError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("CollateralError - AlreadyInLiquidation: The collateral is already in liquidation $0")]
    AlreadyInLiquidation(crate::LiquidationId),
    #[error("CollateralError - InAnotherLiquidation: The collateral is in another liquidation $0")]
    InAnotherLiquidation(crate::LiquidationId),
    #[error("CollateralError - NotInLiquidation: The collateral is not in liquidation")]
    NotInLiquidation,
}

impl ErrorSeverity for CollateralError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::CreditLedgerError(e) => e.severity(),
            Self::ManualUpdateError => Level::WARN,
            Self::JobError(_) => Level::ERROR,
            Self::AlreadyInLiquidation(_) => Level::WARN,
            Self::InAnotherLiquidation(_) => Level::WARN,
            Self::NotInLiquidation => Level::WARN,
        }
    }
}

es_entity::from_es_entity_error!(CollateralError);
