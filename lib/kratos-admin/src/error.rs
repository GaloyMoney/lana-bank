use thiserror::Error;

#[derive(Error, Debug)]
pub enum KratosAdminError {
    #[error("KratosAdminError - HttpError: {0}")]
    HttpError(String),
    #[error("KratosAdminError - ReqwestError: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("KratosAdminError - UuidError: {0}")]
    Uuid(#[from] ::uuid::Error),
    #[error("KratosAdminError - ParseError: {0}")]
    ParseError(String),
}
