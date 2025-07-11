use thiserror::Error;

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
    #[error("ReportError - Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("ReportError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("ReportError - StorageError: {0}")]
    StorageError(#[from] cloud_storage::error::StorageError),
    #[error("ReportError - ApiError: {0}")]
    ApiError(String),
    #[error("ReportError - NotFound")]
    NotFound,
}

es_entity::from_es_entity_error!(ReportError);
