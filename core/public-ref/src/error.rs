use thiserror::Error;

#[derive(Error, Debug)]
pub enum PublicRefError {
    #[error("PublicRefError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PublicRefError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("PublicRefError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
}

es_entity::from_es_entity_error!(PublicRefError);