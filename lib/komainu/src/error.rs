use thiserror::Error;

#[derive(Error, Debug)]
pub enum KomainuError {
    #[error("KomainuError - ReqwestError: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("KomainuError - KomainuError: {error_code}")]
    KomainuError {
        error_code: String,
        errors: Vec<String>,
        status: u16,
    },
}
