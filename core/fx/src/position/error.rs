use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

pub use super::repo::{
    FxPositionCreateError, FxPositionFindError, FxPositionModifyError, FxPositionQueryError,
};

#[derive(Error, Debug)]
pub enum FxPositionError {
    #[error("FxPositionError - InvalidAmount: amount must be positive")]
    InvalidAmount,
    #[error("FxPositionError - InsufficientBalance")]
    InsufficientBalance,
    #[error("FxPositionError - Create: {0}")]
    Create(#[from] FxPositionCreateError),
    #[error("FxPositionError - Modify: {0}")]
    Modify(#[from] FxPositionModifyError),
    #[error("FxPositionError - Find: {0}")]
    Find(#[from] FxPositionFindError),
    #[error("FxPositionError - Query: {0}")]
    Query(#[from] FxPositionQueryError),
    #[error("FxPositionError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl ErrorSeverity for FxPositionError {
    fn severity(&self) -> Level {
        match self {
            Self::InvalidAmount => Level::WARN,
            Self::InsufficientBalance => Level::WARN,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::Sqlx(_) => Level::ERROR,
        }
    }
}
