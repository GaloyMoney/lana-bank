use thiserror::Error;

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
