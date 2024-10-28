use thiserror::Error;

use super::entity::JobType;

#[derive(Error, Debug)]
pub enum JobError {
    #[error("JobError - Sqlx: {0}")]
    Sqlx(sqlx::Error),
    #[error("JobError - InvalidPollInterval: {0}")]
    InvalidPollInterval(String),
    #[error("JobError - InvalidJobType: expected '{0}' but initializer was '{1}'")]
    JobTypeMismatch(JobType, JobType),
    #[error("JobError - JobInitError: {0}")]
    JobInitError(String),
    #[error("JobError - BadState: {0}")]
    CouldNotSerializeExecutionState(serde_json::Error),
    #[error("JobError - BadConfig: {0}")]
    CouldNotSerializeConfig(serde_json::Error),
    #[error("JobError - NoInitializerPresent")]
    NoInitializerPresent,
    #[error("JobError - JobExecutionError: {0}")]
    JobExecutionError(String),
    #[error("JobError - DuplicateId")]
    DuplicateId,
    #[error("JobError - NotFound")]
    NotFound,
}

impl From<Box<dyn std::error::Error>> for JobError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        JobError::JobExecutionError(error.to_string())
    }
}

impl From<sqlx::Error> for JobError {
    fn from(error: sqlx::Error) -> Self {
        if let Some(err) = error.as_database_error() {
            if let Some(constraint) = err.constraint() {
                if constraint.contains("id") {
                    return Self::DuplicateId;
                }
            }
        }
        Self::Sqlx(error)
    }
}

impl From<es_entity::EsEntityError> for JobError {
    fn from(e: es_entity::EsEntityError) -> Self {
        match e {
            es_entity::EsEntityError::NotFound => JobError::NotFound,
            es_entity::EsEntityError::UninitializedFieldError(e) => {
                panic!("Inconsistent data when initializing a Job entity: {:?}", e)
            }
        }
    }
}
