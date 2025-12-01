use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum SmtpError {
    #[error("SmtpError - Transport: {0}")]
    Transport(#[from] lettre::transport::smtp::Error),
    #[error("SmtpError - Lettre: {0}")]
    Lettre(#[from] lettre::error::Error),
    #[error("SmtpError - Address: {0}")]
    Address(#[from] lettre::address::AddressError),
}

impl ErrorSeverity for SmtpError {
    fn severity(&self) -> Level {
        match self {
            Self::Transport(_) => Level::ERROR,
            Self::Lettre(_) => Level::ERROR,
            Self::Address(_) => Level::ERROR,
        }
    }
}
