use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("UserError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("UserError - EntityError: {0}")]
    EntityError(#[from] crate::entity::EntityError),
    #[error("CustomerError - NotFound")]
    NotFound,
    #[error("UserError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
    #[error("UserError - AuditError: {0}")]
    AuditError(#[from] crate::audit::error::AuditError),
    #[error("UserError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
}

impl From<es_entity::EsEntityError> for UserError {
    fn from(e: es_entity::EsEntityError) -> Self {
        match e {
            es_entity::EsEntityError::NotFound => UserError::NotFound,
            es_entity::EsEntityError::UninitializedFieldError(e) => {
                panic!("Inconsistent data when initializing a User entity: {:?}", e)
            }
        }
    }
}
