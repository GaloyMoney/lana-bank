use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustodianStateError {
    #[error("CustodianStateError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CustodianStateError - JsonDecodeError: {0}")]
    JsonDecode(#[from] serde_json::Error),
}
