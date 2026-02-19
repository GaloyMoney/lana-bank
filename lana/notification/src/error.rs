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
    #[error("NotificationError - DomainConfig: {0}")]
    DomainConfig(#[from] domain_config::DomainConfigError),
    #[error("NotificationError - Authorization: {0}")]
    Authorization(#[from] authz::error::AuthorizationError),
    #[error("NotificationError - RegisterEventHandler: {0}")]
    RegisterEventHandler(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl ErrorSeverity for NotificationError {
    fn severity(&self) -> Level {
        match self {
            Self::Email(e) => e.severity(),
            Self::Job(_) => Level::ERROR,
            Self::DomainConfig(e) => e.severity(),
            Self::Authorization(e) => e.severity(),
            Self::RegisterEventHandler(_) => Level::ERROR,
        }
    }
}
