use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

pub use super::repo::{RoleCreateError, RoleFindError, RoleModifyError, RoleQueryError};

#[derive(Error, Debug)]
pub enum RoleError {
    #[error("RoleError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("RoleError - Create: {0}")]
    Create(#[from] RoleCreateError),
    #[error("RoleError - Modify: {0}")]
    Modify(#[from] RoleModifyError),
    #[error("RoleError - Find: {0}")]
    Find(#[from] RoleFindError),
    #[error("RoleError - Query: {0}")]
    Query(#[from] RoleQueryError),
    #[error("RoleError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("RoleError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
}

impl ErrorSeverity for RoleError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
        }
    }
}
