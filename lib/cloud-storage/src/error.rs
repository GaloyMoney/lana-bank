use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Storage Error - StorageClientError: {0}")]
    StorageClientError(#[from] super::client::error::StorageClientError),
}
