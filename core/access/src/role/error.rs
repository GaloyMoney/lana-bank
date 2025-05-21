use thiserror::Error;

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
