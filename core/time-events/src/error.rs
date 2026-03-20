use thiserror::Error;

use tracing_utils::{ErrorSeverity, Level};

#[derive(Error, Debug)]
pub enum TimeEventsError {
    #[error("TimeEventsError - DomainConfigError: {0}")]
    DomainConfig(#[from] domain_config::DomainConfigError),
    #[error("TimeEventsError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("TimeEventsError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("TimeEventsError - TimeAdvanceUnavailable")]
    TimeAdvanceUnavailable,
    #[error("TimeEventsError - TimeAdvanceFailed: {0}")]
    TimeAdvanceFailed(String),
}

impl ErrorSeverity for TimeEventsError {
    fn severity(&self) -> Level {
        match self {
            Self::DomainConfig(_) => Level::ERROR,
            Self::JobError(_) => Level::ERROR,
            Self::AuthorizationError(_) => Level::WARN,
            Self::TimeAdvanceUnavailable => Level::WARN,
            Self::TimeAdvanceFailed(_) => Level::WARN,
        }
    }
}
