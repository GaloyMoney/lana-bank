use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::primitives::{LiquidationId, Satoshis};

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
    #[error("CollateralError - LiquidationError: {0}")]
    LiquidationError(#[from] crate::liquidation::error::LiquidationError),
    #[error(
        "CollateralError - InsufficientCollateral: requested {requested}, available {available}"
    )]
    InsufficientCollateral {
        requested: Satoshis,
        available: Satoshis,
    },
    #[error("CollateralError - NoCollateralToLiquidate: collateral amount is zero")]
    NoCollateralToLiquidate,
    #[error(
        "CollateralError - LiquidationAlreadyActive: cannot start a new liquidation while one is active"
    )]
    LiquidationAlreadyActive,
    #[error("CollateralError - LiquidationNotFound: {0}")]
    LiquidationNotFound(LiquidationId),
    #[error("CollateralError - NoActiveLiquidation: no active liquidation for this collateral")]
    NoActiveLiquidation,
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
            Self::JobError(_) => Level::ERROR,
            Self::InsufficientCollateral { .. } => Level::WARN,
            Self::NoCollateralToLiquidate => Level::WARN,
            Self::LiquidationAlreadyActive => Level::WARN,
            Self::LiquidationNotFound(_) => Level::ERROR,
            Self::LiquidationError(e) => e.severity(),
            Self::NoActiveLiquidation => Level::WARN,
            Self::AuthorizationError(e) => e.severity(),
            Self::LedgerTransactionInitiatorParseError(e) => e.severity(),
        }
    }
}

es_entity::from_es_entity_error!(CollateralError);
