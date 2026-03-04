use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::repo::{UserCreateError, UserFindError, UserModifyError, UserQueryError};

#[derive(Error, Debug)]
pub enum UserError {
    #[error("UserError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("UserError - Create: {0}")]
    Create(#[from] UserCreateError),
    #[error("UserError - Modify: {0}")]
    Modify(#[from] UserModifyError),
    #[error("UserError - Find: {0}")]
    Find(#[from] UserFindError),
    #[error("UserError - Query: {0}")]
    Query(#[from] UserQueryError),
    #[error("UserError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("UserError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("SubjectError - SubjectIsNotUser")]
    SubjectIsNotUser,
}

impl ErrorSeverity for UserError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::SubjectIsNotUser => Level::WARN,
        }
    }
}
