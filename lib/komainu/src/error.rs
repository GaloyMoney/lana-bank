use thiserror::Error;

#[derive(Error, Debug)]
pub enum KomainuError {
    #[error("KomainuError - ReqwestError: {0}")]
    ReqwestError(#[from] reqwest::Error),
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
