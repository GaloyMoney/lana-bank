use opentelemetry_sdk::{error::OTelSdkError, trace::TraceError};
use thiserror::Error;
use tracing::Level;

use crate::error_severity::ErrorSeverity;

#[derive(Error, Debug)]
pub enum TracingError {
    #[error("TracingError - TracerProvider: {0}")]
    TracerProvider(#[from] TraceError),
    #[error("TracingError - OtelSdk: {0}")]
    OtelSdk(#[from] OTelSdkError),
}

impl ErrorSeverity for TracingError {
    fn severity(&self) -> Level {
        match self {
            Self::TracerProvider(_) => Level::ERROR,
            Self::OtelSdk(_) => Level::ERROR,
        }
    }
}
