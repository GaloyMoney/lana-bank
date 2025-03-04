use thiserror::Error;

use super::smtp::error::SmtpError;

#[derive(Error, Debug)]
pub enum EmailError {
    #[error("EmailError - SmtpError: {0}")]
    Smtp(#[from] SmtpError),
    #[error("EmailError - Template: {0}")]
    Template(String),
    #[error("EmailError - Job: {0}")]
    Job(#[from] ::job::error::JobError),
    #[error("EmailError - Database: {0}")]
    Database(#[from] sqlx::Error),
    #[error("EmailError - Outbox: {0}")]
    Outbox(String),
}
