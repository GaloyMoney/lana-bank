use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Failed to authenticate: {0}")]
    Auth(#[from] google_cloud_storage::client::google_cloud_auth::error::Error),
    #[error("Google Cloud Storage error: {0}")]
    Gcs(#[from] google_cloud_storage::http::Error),
    #[error("Failed to sign URL: {0}")]
    GcsSignUrl(#[from] google_cloud_storage::sign::SignedURLError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("bucket name not configured")]
    MissingBucket,
}
