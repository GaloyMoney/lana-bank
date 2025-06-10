use thiserror::Error;

use crate::email::error::EmailError;
use ::job::error::JobError;

#[derive(Error, Debug)]
pub enum NotificationError {
    #[error("NotificationError - Email: {0}")]
    Email(#[from] EmailError),
    #[error("NotificationError - Job: {0}")]
    Job(#[from] JobError),
}
