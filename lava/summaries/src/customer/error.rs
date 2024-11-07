use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomerSummaryError {
    #[error("CustomerSummaryError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CustomerSummaryError - JobError: {0}")]
    Job(#[from] ::job::error::JobError),
    #[error("CustomerSummaryError - Authorization: {0}")]
    Authorization(#[from] authz::error::AuthorizationError),
}
