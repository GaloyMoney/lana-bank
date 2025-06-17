use thiserror::Error;

#[derive(Error, Debug)]
pub enum DocumentStorageError {
    #[error("DocumentStorageError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DocumentStorageError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("DocumentStorageError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("DocumentStorageError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("DocumentStorageError - AuditError: ${0}")]
    AuditError(#[from] audit::error::AuditError),
}

es_entity::from_es_entity_error!(DocumentStorageError);