use thiserror::Error;

#[derive(Error, Debug)]
pub enum PublicIdError {
    #[error("PublicIdError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PublicIdError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("PublicIdError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
}

es_entity::from_es_entity_error!(PublicIdError);
