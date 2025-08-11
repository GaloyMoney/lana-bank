use thiserror::Error;

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
    #[error("BitgoError - InvalidWebhookUrl")]
    InvalidWebhookUrl,
    #[error("BitgoError - DecryptXprvError: {0}")]
    DecryptXprv(String),
    #[error("BitgoError - MissingWebhookSignature")]
    MissingWebhookSignature,
    #[error("BitgoError - InvalidWebhookSignature")]
    InvalidWebhookSignature(#[from] sha2::digest::MacError),
}
