use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum ReportError {
    #[error("ReportError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ReportError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("ReportError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("ReportError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("ReportError - AuditError: ${0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("ReportError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("ReportError - StorageError: {0}")]
    StorageError(#[from] cloud_storage::error::StorageError),
    #[error("ReportError - ReportError: {0}")]
    ReportError(#[from] crate::report::error::ReportError),
    #[error("ReportError - ReportRunError: {0}")]
    ReportRunError(#[from] crate::report_run::error::ReportRunError),
    #[error("ReportError - Disabled")]
    Disabled,
    #[error("ReportError - NotFound")]
    NotFound,
}

es_entity::from_es_entity_error!(ReportError);

impl ErrorSeverity for ReportError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::JobError(_) => Level::ERROR,
            Self::StorageError(e) => e.severity(),
            Self::ReportError(e) => e.severity(),
            Self::ReportRunError(e) => e.severity(),
            Self::Disabled => Level::WARN,
            Self::NotFound => Level::WARN,
        }
    }
}
