use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum BitfinexError {
    #[error("BitfinexError - ReqwestError: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("BitfinexError - Unexpected JSON format: {0}")]
    JsonFormat(#[from] serde_json::Error),
    #[error("BitfinexError - BitfinexApiError: {message}")]
    BitfinexApiError { message: String },
    #[error("BitfinexError - UnexpectedResponseFormat: {0}")]
    UnexpectedResponseFormat(String),
}

impl ErrorSeverity for BitfinexError {
    fn severity(&self) -> Level {
        match self {
            Self::ReqwestError(_) => Level::ERROR,
            Self::JsonFormat(_) => Level::ERROR,
            Self::BitfinexApiError { .. } => Level::ERROR,
            Self::UnexpectedResponseFormat(_) => Level::ERROR,
        }
    }
}
