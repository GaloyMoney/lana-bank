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
}

es_entity::from_es_entity_error!(ReportError);
