use thiserror::Error;

use crate::primitives::ReportId;

#[derive(Error, Debug)]
pub enum ReportError {
    #[error("ReportError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ReportError - EntityError: {0}")]
    EntityError(#[from] crate::entity::EntityError),
    #[error("ReportError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
    #[error("ReportError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
    #[error("ReportError - CouldNotFindById: {0}")]
    CouldNotFindById(ReportId),
}
