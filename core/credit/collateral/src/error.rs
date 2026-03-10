use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::repo::{
    CollateralCreateError, CollateralFindError, CollateralModifyError, CollateralQueryError,
    LiquidationQueryError,
};

#[derive(thiserror::Error, Debug)]
pub enum CollateralError {
    #[error("CollateralError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CollateralError - Create: {0}")]
    Create(#[from] CollateralCreateError),
    #[error("CollateralError - Modify: {0}")]
    Modify(#[from] CollateralModifyError),
    #[error("CollateralError - Find: {0}")]
    Find(#[from] CollateralFindError),
    #[error("CollateralError - Query: {0}")]
    Query(#[from] CollateralQueryError),
    #[error("CollateralError - CollateralLedgerError: {0}")]
    CollateralLedgerError(#[from] super::ledger::CollateralLedgerError),
    #[error("CollateralError - ManualUpdateError: Cannot update collateral with a custodian")]
    ManualUpdateError,
    #[error("CollateralError - NoActiveLiquidation")]
    NoActiveLiquidation,
    #[error("CollateralError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("CollateralError - RegisterEventHandler: {0}")]
    RegisterEventHandler(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("CollateralError - LiquidationError: {0}")]
    LiquidationError(#[from] super::liquidation::LiquidationError),
    #[error("CollateralError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CollateralError - ManualCustodianDisabled")]
    ManualCustodianDisabled,
    #[error("CollateralError - DomainConfigError: {0}")]
    DomainConfigError(#[from] domain_config::error::DomainConfigError),
}

impl From<LiquidationQueryError> for CollateralError {
    fn from(e: LiquidationQueryError) -> Self {
        Self::LiquidationError(e.into())
    }
}

impl ErrorSeverity for CollateralError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::CollateralLedgerError(e) => e.severity(),
            Self::ManualUpdateError => Level::WARN,
            Self::NoActiveLiquidation => Level::WARN,
            Self::JobError(_) => Level::ERROR,
            Self::RegisterEventHandler(_) => Level::ERROR,
            Self::LiquidationError(e) => e.severity(),
            Self::AuthorizationError(e) => e.severity(),
            Self::ManualCustodianDisabled => Level::WARN,
            Self::DomainConfigError(_) => Level::ERROR,
        }
    }
}
