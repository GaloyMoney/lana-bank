use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomerInfoError {
    #[error("CustomerInfoError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CustomerInfoError - JobError: {0}")]
    Job(#[from] ::job::error::JobError),
    #[error("CustomerInfoError - Authorization: {0}")]
    Authorization(#[from] authz::error::AuthorizationError),
}
