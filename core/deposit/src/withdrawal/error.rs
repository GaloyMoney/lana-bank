use thiserror::Error;

#[derive(Error, Debug)]
pub enum WithdrawalError {
    #[error("WithdrawalError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("WithdrawalError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("WithdrawalError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
}

es_entity::from_es_entity_error!(WithdrawalError);
