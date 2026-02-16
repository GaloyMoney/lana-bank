use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("EncryptionError - Decryption")]
    Decryption,
}

impl ErrorSeverity for EncryptionError {
    fn severity(&self) -> Level {
        match self {
            Self::Decryption => Level::ERROR,
        }
    }
}
