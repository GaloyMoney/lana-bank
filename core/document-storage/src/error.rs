use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum DocumentStorageError {
    #[error("DocumentStorageError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DocumentStorageError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("DocumentStorageError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("DocumentStorageError - StorageError: {0}")]
    StorageError(#[from] cloud_storage::error::StorageError),
}

es_entity::from_es_entity_error!(DocumentStorageError);

impl ErrorSeverity for DocumentStorageError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::StorageError(e) => e.severity(),
        }
    }
}
