use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum StorageClientError {
    #[error("Google Cloud Storage error: {0}")]
    Gcs(#[from] google_cloud_storage::Error),
    #[error("Failed to sign URL: {0}")]
    GcsSignUrl(#[from] google_cloud_storage::error::SigningError),
    #[error("Failed to write object: {0}")]
    GcsWrite(#[from] google_cloud_storage::error::WriteError),
    #[error("Failed to initialize storage client: {0}")]
    Init(Box<dyn std::error::Error + Send + Sync>),
    #[error("StorageClientError - StdIo: {0}")]
    StdIo(#[from] std::io::Error),
    #[error("Signed URL has expired")]
    SignatureExpired,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid path")]
    InvalidPath,
    #[error("StorageClientError - Other: {0}")]
    Other(#[from] anyhow::Error),
}

impl ErrorSeverity for StorageClientError {
    fn severity(&self) -> Level {
        match self {
            Self::Gcs(_) => Level::ERROR,
            Self::GcsSignUrl(_) => Level::ERROR,
            Self::GcsWrite(_) => Level::ERROR,
            Self::Init(_) => Level::ERROR,
            Self::StdIo(_) => Level::ERROR,
            Self::SignatureExpired => Level::WARN,
            Self::InvalidSignature => Level::WARN,
            Self::InvalidPath => Level::WARN,
            Self::Other(_) => Level::ERROR,
        }
    }
}
