use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreUserError {
    #[error("CoreUserError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CoreUserError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("CoreUserError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("CoreUserError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CoreUserError - UserError: {0}")]
    UserError(#[from] super::user::UserError),
    #[error("CoreUserError - RoleError: {0}")]
    RoleError(#[from] super::role::RoleError),
    #[error("CoreUserError - PermissionSetError: {0}")]
    PermissionSetError(#[from] super::permission_set::PermissionSetError),
}
