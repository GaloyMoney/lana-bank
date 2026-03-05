use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::repo::{
    LiquidationCreateError, LiquidationFindError, LiquidationModifyError, LiquidationQueryError,
};

#[derive(Error, Debug, strum::IntoStaticStr)]
pub enum LiquidationError {
    #[error("LiquidationError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("LiquidationError - Create: {0}")]
    Create(#[from] LiquidationCreateError),
    #[error("LiquidationError - Modify: {0}")]
    Modify(#[from] LiquidationModifyError),
    #[error("LiquidationError - Find: {0}")]
    Find(#[from] LiquidationFindError),
    #[error("LiquidationError - Query: {0}")]
    Query(#[from] LiquidationQueryError),
    #[error("LiquidationError - AlreadySatisfied")]
    AlreadySatisfied,
    #[error("LiquidationError - AlreadyCompleted")]
    AlreadyCompleted,
    #[error("LiquidationError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("LiquidationError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
}

impl ErrorSeverity for LiquidationError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::AlreadySatisfied | Self::AlreadyCompleted => Level::WARN,
            Self::AuthorizationError(e) => e.severity(),
            Self::JobError(_) => Level::ERROR,
        }
    }
}
