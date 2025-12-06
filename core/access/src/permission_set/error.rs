use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum PermissionSetError {
    #[error("PermissionSetError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PermissionSetError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("PermissionSetError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("PermissionSetError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
}

es_entity::from_es_entity_error!(PermissionSetError);

impl ErrorSeverity for PermissionSetError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
        }
    }
}
