use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

pub use super::repo::{
    PermissionSetCreateError, PermissionSetFindError, PermissionSetModifyError,
    PermissionSetQueryError,
};

#[derive(Error, Debug)]
pub enum PermissionSetError {
    #[error("PermissionSetError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PermissionSetError - Create: {0}")]
    Create(#[from] PermissionSetCreateError),
    #[error("PermissionSetError - Modify: {0}")]
    Modify(#[from] PermissionSetModifyError),
    #[error("PermissionSetError - Find: {0}")]
    Find(#[from] PermissionSetFindError),
    #[error("PermissionSetError - Query: {0}")]
    Query(#[from] PermissionSetQueryError),
    #[error("PermissionSetError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
}

impl ErrorSeverity for PermissionSetError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
        }
    }
}
