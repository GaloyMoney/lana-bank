use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum BitgoError {
    #[error("BitgoError - ReqwestError: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("BitgoError - Unexpected JSON format: {0}")]
    JsonFormat(#[from] serde_json::Error),
    #[error("BitgoError - BitgoError: {name}: {error}")]
    BitgoError {
        error: String,
        name: String,
        request_id: String,
    },
    #[error("BitgoError - DecryptXprvError: {0}")]
    DecryptXprv(String),
    #[error("BitgoError - MissingWebhookSignature")]
    MissingWebhookSignature,
    #[error("BitgoError - InvalidWebhookSignature")]
    InvalidWebhookSignature(#[from] sha2::digest::MacError),
}

impl ErrorSeverity for BitgoError {
    fn severity(&self) -> Level {
        match self {
            Self::ReqwestError(_) => Level::ERROR,
            Self::JsonFormat(_) => Level::ERROR,
            Self::BitgoError { .. } => Level::ERROR,
            Self::DecryptXprv(_) => Level::ERROR,
            Self::MissingWebhookSignature => Level::WARN,
            Self::InvalidWebhookSignature(_) => Level::WARN,
        }
    }
}
