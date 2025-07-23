use thiserror::Error;

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
