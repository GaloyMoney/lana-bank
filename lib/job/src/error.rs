use thiserror::Error;

use super::entity::JobType;

#[derive(Error, Debug)]
pub enum JobError {
    #[error("JobError - Sqlx: {0}")]
    Sqlx(sqlx::Error),
    #[error("JobError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("JobError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
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
    #[error("JobError - DuplicateUniqueJobType")]
    DuplicateUniqueJobType,
}

es_entity::from_es_entity_error!(JobError);

#[derive(Debug)]
pub enum JobRunError {
    Transient(Box<dyn std::error::Error + Send + Sync>),
    Permanent(Box<dyn std::error::Error + Send + Sync>),
}

impl std::fmt::Display for JobRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transient(e) | Self::Permanent(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for JobRunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Transient(e) | Self::Permanent(e) => Some(&**e),
        }
    }
}

impl<E> From<E> for JobRunError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(e: E) -> Self {
        Self::Transient(Box::new(e))
    }
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
                if constraint.contains("type") {
                    return Self::DuplicateUniqueJobType;
                }
                if constraint.contains("id") {
                    return Self::DuplicateId;
                }
            }
        }
        Self::Sqlx(error)
    }
}
