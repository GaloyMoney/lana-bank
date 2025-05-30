use thiserror::Error;

#[derive(Error, Debug)]
pub enum KomainuError {
    #[error("KomainuError - ReqwestError: {0}")]
    ReqwestError(#[from] reqwest::Error),
}
