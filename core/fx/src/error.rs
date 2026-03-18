use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CoreFxError {
    #[error("CoreFxError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CoreFxError - ChartLookupError: {0}")]
    ChartLookupError(#[from] chart_primitives::ChartLookupError),
    #[error("CoreFxError - DomainConfigError: {0}")]
    DomainConfigError(#[from] domain_config::DomainConfigError),
}

impl ErrorSeverity for CoreFxError {
    fn severity(&self) -> Level {
        match self {
            Self::AuthorizationError(e) => e.severity(),
            Self::ChartLookupError(e) => e.severity(),
            Self::DomainConfigError(e) => e.severity(),
        }
    }
}
