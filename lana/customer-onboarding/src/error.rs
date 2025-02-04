use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomerOnboardingError {
    #[error("CustomerOnboardingError - JobError: {0}")]
    Job(#[from] ::job::error::JobError),
}
