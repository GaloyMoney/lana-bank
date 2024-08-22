use thiserror::Error;

use crate::primitives::UserId;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("UserError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("UserError - EntityError: {0}")]
    EntityError(#[from] crate::entity::EntityError),
    #[error("UserError - CouldNotFindByEmail: {0}")]
    CouldNotFindByEmail(String),
    #[error("UserError - CouldNotFindById: {0}")]
    CouldNotFindById(UserId),
    #[error("UserError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
    #[error("UserError - AuditError: {0}")]
    AuditError(#[from] crate::audit::error::AuditError),
}
