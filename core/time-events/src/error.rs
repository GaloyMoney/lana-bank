use thiserror::Error;

use tracing_utils::{ErrorSeverity, Level};

#[derive(Error, Debug, strum::IntoStaticStr)]
pub enum TimeEventsError {
    #[error("TimeEventsError - DomainConfigError: {0}")]
    DomainConfig(#[from] domain_config::DomainConfigError),
    #[error("TimeEventsError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
}

impl ErrorSeverity for TimeEventsError {
    fn severity(&self) -> Level {
        match self {
            Self::DomainConfig(_) => Level::ERROR,
            Self::JobError(_) => Level::ERROR,
        }
    }

    fn variant_name(&self) -> &'static str {
        self.into()
    }
}
