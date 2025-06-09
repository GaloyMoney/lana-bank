use thiserror::Error;

#[derive(Debug, Error)]
pub enum CustodianClientError {
    #[error("CustodianClientError - ClientError: {0}")]
    ClientError(#[from] Box<dyn std::error::Error + Send + Sync>),
}
