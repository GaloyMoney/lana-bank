use thiserror::Error;

#[derive(Error, Debug)]
pub enum PaymentError {
    #[error("PaymentError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PaymentError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("PaymentError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
}

es_entity::from_es_entity_error!(PaymentError);
