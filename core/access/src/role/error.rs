use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum RoleError {
    #[error("RoleError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("RoleError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("RoleError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("RoleError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("RoleError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
}

es_entity::from_es_entity_error!(RoleError);

impl ErrorSeverity for RoleError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
        }
    }
}
