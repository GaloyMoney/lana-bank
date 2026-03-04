use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::repo::{
    PublicIdEntityCreateError, PublicIdEntityFindError, PublicIdEntityModifyError,
    PublicIdEntityQueryError,
};

#[derive(Error, Debug)]
pub enum PublicIdError {
    #[error("PublicIdError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PublicIdError - Create: {0}")]
    Create(#[from] PublicIdEntityCreateError),
    #[error("PublicIdError - Modify: {0}")]
    Modify(#[from] PublicIdEntityModifyError),
    #[error("PublicIdError - Find: {0}")]
    Find(#[from] PublicIdEntityFindError),
    #[error("PublicIdError - Query: {0}")]
    Query(#[from] PublicIdEntityQueryError),
}

impl ErrorSeverity for PublicIdError {
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
