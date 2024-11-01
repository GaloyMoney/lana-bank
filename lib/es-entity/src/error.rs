use thiserror::Error;

#[derive(Error, Debug)]
pub enum EsEntityError {
    #[error("EsEntityError - UninitializedFieldError: {0}")]
    UninitializedFieldError(#[from] derive_builder::UninitializedFieldError),
    #[error("EsEntityError - Deserialization: {0}")]
    EventDeserialization(#[from] serde_json::Error),
    #[error("EntityError - NotFound")]
    NotFound,
    #[error("EntityError - ConcurrentModification")]
    ConcurrentModification,
    #[error("EntityError - InconsistentIdempotency")]
    InconsistenmtIdempotency,
}

#[derive(Error, Debug)]
pub enum EsRepoError {
    #[error("EsRepoError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("{0}")]
    EsEntityError(EsEntityError),
}
crate::from_es_entity_error!(EsRepoError);
