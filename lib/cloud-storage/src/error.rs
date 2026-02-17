use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Storage Error - StorageClientError: {0}")]
    StorageClientError(#[from] super::client::error::StorageClientError),
    #[error("Local storage is not configured")]
    LocalStorageNotConfigured,
}

impl ErrorSeverity for StorageError {
    fn severity(&self) -> Level {
        match self {
            Self::StorageClientError(e) => e.severity(),
            Self::LocalStorageNotConfigured => Level::ERROR,
        }
    }
}
