use thiserror::Error;

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
