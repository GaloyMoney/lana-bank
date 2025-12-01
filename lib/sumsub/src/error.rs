use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum SumsubError {
    #[error("SumsubError - ReqwestError: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("SumsubError - JSON format error: {0}")]
    JsonFormat(#[from] serde_json::Error),
    #[error("SumsubError - SystemTimeError: {0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),
    #[error("SumsubError - InvalidHeaderValue: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("SumsubError - API Error: {code}, {description}")]
    ApiError { code: u16, description: String },
    #[error("SumsubError - InvalidResponse: {0}")]
    InvalidResponse(String),
}

impl ErrorSeverity for SumsubError {
    fn severity(&self) -> Level {
        match self {
            Self::ReqwestError(_) => Level::ERROR,
            Self::JsonFormat(_) => Level::ERROR,
            Self::SystemTimeError(_) => Level::ERROR,
            Self::InvalidHeaderValue(_) => Level::ERROR,
            Self::ApiError { code, .. } => {
                // 4xx errors might be less severe than 5xx errors
                if *code >= 500 {
                    Level::ERROR
                } else if *code >= 400 {
                    Level::WARN
                } else {
                    Level::ERROR
                }
            }
            Self::InvalidResponse(_) => Level::ERROR,
        }
    }
}
