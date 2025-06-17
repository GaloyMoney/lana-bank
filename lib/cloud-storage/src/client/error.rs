use thiserror::Error;

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
