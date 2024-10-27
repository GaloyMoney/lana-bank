use thiserror::Error;

#[derive(Error, Debug)]
pub enum DocumentError {
    #[error("DocumentError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DocumentError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
    #[error("DocumentError - StorageError: {0}")]
    StorageError(#[from] crate::storage::StorageError),
    #[error("DocumentError - NotFound")]
    NotFound,
}

impl From<es_entity::EsEntityError> for DocumentError {
    fn from(e: es_entity::EsEntityError) -> Self {
        match e {
            es_entity::EsEntityError::NotFound => DocumentError::NotFound,
            es_entity::EsEntityError::UninitializedFieldError(e) => {
                panic!(
                    "Inconsistent data when initializing a Document entity: {:?}",
                    e
                )
            }
        }
    }
}
