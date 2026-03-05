use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::repo::{
    DocumentCreateError, DocumentFindError, DocumentModifyError, DocumentQueryError,
};

#[derive(Error, Debug, strum::IntoStaticStr)]
pub enum DocumentStorageError {
    #[error("DocumentStorageError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DocumentStorageError - Create: {0}")]
    Create(#[from] DocumentCreateError),
    #[error("DocumentStorageError - Modify: {0}")]
    Modify(#[from] DocumentModifyError),
    #[error("DocumentStorageError - Find: {0}")]
    Find(#[from] DocumentFindError),
    #[error("DocumentStorageError - Query: {0}")]
    Query(#[from] DocumentQueryError),
    #[error("DocumentStorageError - StorageError: {0}")]
    StorageError(#[from] cloud_storage::error::StorageError),
}

impl ErrorSeverity for DocumentStorageError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::StorageError(e) => e.severity(),
        }
    }
}
