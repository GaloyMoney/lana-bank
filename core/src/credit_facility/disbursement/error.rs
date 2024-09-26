use thiserror::Error;

#[derive(Error, Debug)]
pub enum DisbursementError {
    #[error("DisbursementError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DisbursementError - EntityError: {0}")]
    EntityError(#[from] crate::entity::EntityError),
    #[error("DisbursementError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
    #[error("DisbursementError - UserCannotApproveTwice")]
    UserCannotApproveTwice,
    #[error("DisbursementError - AlreadyApproved")]
    AlreadyApproved,
}
