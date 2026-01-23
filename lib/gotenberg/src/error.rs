use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum GotenbergError {
    #[error("GotenbergError - HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("GotenbergError - Multipart error: {0}")]
    Multipart(String),
    #[error("GotenbergError - Server error: {0}")]
    Server(String),
}

impl ErrorSeverity for GotenbergError {
    fn severity(&self) -> Level {
        match self {
            Self::Http(_) => Level::ERROR,
            Self::Multipart(_) => Level::ERROR,
            Self::Server(_) => Level::ERROR,
        }
    }
}
