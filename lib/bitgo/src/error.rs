use thiserror::Error;

#[derive(Error, Debug)]
pub enum BitgoError {
    #[error("BitgoError - ReqwestError: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("BitgoError - InvalidEndpoint: {0}")]
    InvalidEndpoint(String),
    #[error("BitgoError - Unexpected JSON format: {0}")]
    JsonFormat(#[from] serde_json::Error),
    #[error("BitgoError - MissingWebhookSignature")]
    MissingWebhookSignature,
    #[error("BitgoError - InvalidWebhookSignature")]
    InvalidWebhookSignature(#[from] sha2::digest::MacError),
}
