use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum KomainuError {
    #[error("KomainuError - ReqwestError: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("KomainuError - ConfigurationError: Could not parse secret key")]
    SecretKey,
    #[error("KomainuError - KomainuError: {error_code}")]
    KomainuError {
        error_code: String,
        errors: Vec<String>,
        status: u16,
    },
    #[error("BitgoError - Unexpected JSON format: {0}")]
    JsonFormat(#[from] serde_json::Error),
    #[error("Error - MissingWebhookHeaders")]
    MissingWebhookHeaders,
    #[error("KomainuError - InvalidWebhookSignature")]
    InvalidWebhookSignature(#[from] sha2::digest::MacError),
}

impl ErrorSeverity for KomainuError {
    fn severity(&self) -> Level {
        match self {
            Self::ReqwestError(_) => Level::ERROR,
            Self::SecretKey => Level::WARN,
            Self::KomainuError { .. } => Level::ERROR,
            Self::JsonFormat(_) => Level::ERROR,
            Self::MissingWebhookHeaders => Level::WARN,
            Self::InvalidWebhookSignature(_) => Level::WARN,
        }
    }
}
