use thiserror::Error;

#[derive(Error, Debug)]
pub enum BitgoError {
    #[error("KomainuError - ReqwestError: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("KomainuError - InvalidEndpoint: {0}")]
    InvalidEndpoint(String),
}
