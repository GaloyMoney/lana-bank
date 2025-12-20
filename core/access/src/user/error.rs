use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("UserError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("UserError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("UserError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("UserError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("UserError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("SubjectError - SubjectIsNotUser")]
    SubjectIsNotUser,
}

es_entity::from_es_entity_error!(UserError);

impl ErrorSeverity for UserError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::SubjectIsNotUser => Level::WARN,
        }
    }
}
