use thiserror::Error;

#[derive(Error, Debug)]
pub enum SmtpError {
    #[error("SmtpError - Transport: {0}")]
    Transport(String),
    #[error("SmtpError - Lettre: {0}")]
    Lettre(#[from] lettre::error::Error),
    #[error("SmtpError - Address: {0}")]
    Address(#[from] lettre::address::AddressError),
}
