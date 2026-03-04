use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::repo::{DepositCreateError, DepositFindError, DepositModifyError, DepositQueryError};

#[derive(Error, Debug)]
pub enum DepositError {
    #[error("DepositError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DepositError - Create: {0}")]
    Create(#[from] DepositCreateError),
    #[error("DepositError - Modify: {0}")]
    Modify(#[from] DepositModifyError),
    #[error("DepositError - Find: {0}")]
    Find(#[from] DepositFindError),
    #[error("DepositError - Query: {0}")]
    Query(#[from] DepositQueryError),
}

impl ErrorSeverity for DepositError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
        }
    }
}
