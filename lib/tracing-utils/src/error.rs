use opentelemetry_sdk::{error::OTelSdkError, trace::TraceError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TracingError {
    #[error("TracingError - TracerProvider: {0}")]
    TracerProvider(#[from] TraceError),
    #[error("TracingError - OtelSdk: {0}")]
    OtelSdk(#[from] OTelSdkError),
}
