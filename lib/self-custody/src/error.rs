use thiserror::Error;

#[derive(Debug, Error)]
pub enum SelfCustodyError {
    #[error("SelfCustodyError - InvalidXpub: {0}")]
    InvalidXpub(String),
    #[error("SelfCustodyError - DerivationError: {0}")]
    DerivationError(String),
    #[error("SelfCustodyError - NetworkMismatch: expected {expected}, got {actual}")]
    NetworkMismatch { expected: String, actual: String },
    #[error("SelfCustodyError - EsploraRequest: {0}")]
    EsploraRequest(#[from] reqwest::Error),
    #[error("SelfCustodyError - EsploraResponse: {0}")]
    EsploraResponse(String),
}
