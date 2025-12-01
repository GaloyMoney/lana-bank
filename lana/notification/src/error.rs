use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::email::error::EmailError;
use ::job::error::JobError;

#[derive(Error, Debug)]
pub enum NotificationError {
    #[error("NotificationError - Email: {0}")]
    Email(#[from] EmailError),
    #[error("NotificationError - Job: {0}")]
    Job(#[from] JobError),
}

impl ErrorSeverity for NotificationError {
    fn severity(&self) -> Level {
        match self {
            Self::Email(e) => e.severity(),
            Self::Job(_) => Level::ERROR,
        }
    }
}
