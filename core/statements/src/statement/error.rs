use thiserror::Error;

#[derive(Error, Debug)]
pub enum StatementError {
    #[error("StatementError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("StatementError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("StatementError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
}

es_entity::from_es_entity_error!(StatementError);
