use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CoreAccessError {
    #[error("CoreAccessError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CoreAccessError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("CoreAccessError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("CoreAccessError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CoreAccessError - UserError: {0}")]
    UserError(#[from] super::user::UserError),
    #[error("CoreAccessError - RoleError: {0}")]
    RoleError(#[from] super::role::RoleError),
    #[error("CoreAccessError - PermissionSetError: {0}")]
    PermissionSetError(#[from] super::permission_set::PermissionSetError),
}

es_entity::from_es_entity_error!(CoreAccessError);

impl ErrorSeverity for CoreAccessError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::UserError(e) => e.severity(),
            Self::RoleError(e) => e.severity(),
            Self::PermissionSetError(e) => e.severity(),
        }
    }
}
