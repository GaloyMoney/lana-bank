use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

pub use super::repo::{
    ApprovalProcessCreateError, ApprovalProcessFindError, ApprovalProcessModifyError,
    ApprovalProcessQueryError,
};

#[derive(Error, Debug, strum::IntoStaticStr)]
pub enum ApprovalProcessError {
    #[error("ApprovalProcessError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ApprovalProcessError - Create: {0}")]
    Create(#[from] ApprovalProcessCreateError),
    #[error("ApprovalProcessError - Modify: {0}")]
    Modify(#[from] ApprovalProcessModifyError),
    #[error("ApprovalProcessError - Find: {0}")]
    Find(#[from] ApprovalProcessFindError),
    #[error("ApprovalProcessError - Query: {0}")]
    Query(#[from] ApprovalProcessQueryError),
}

impl ErrorSeverity for ApprovalProcessError {
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
