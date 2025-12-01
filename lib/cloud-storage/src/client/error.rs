use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum StorageClientError {
    #[error("Failed to authenticate: {0}")]
    Auth(#[from] google_cloud_storage::client::google_cloud_auth::error::Error),
    #[error("Google Cloud Storage error: {0}")]
    Gcs(#[from] google_cloud_storage::http::Error),
    #[error("Failed to sign URL: {0}")]
    GcsSignUrl(#[from] google_cloud_storage::sign::SignedURLError),
    #[error("StorageClientError - StdIo: {0}")]
    StdIo(#[from] std::io::Error),
}

impl ErrorSeverity for StorageClientError {
    fn severity(&self) -> Level {
        match self {
            Self::Auth(_) => Level::ERROR,
            Self::Gcs(_) => Level::ERROR,
            Self::GcsSignUrl(_) => Level::ERROR,
            Self::StdIo(_) => Level::ERROR,
        }
    }
}
