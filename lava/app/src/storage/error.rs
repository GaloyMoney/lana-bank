use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Cloud Storage Error: {0}")]
    CloudStorage(#[from] cloud_storage::Error),
}
