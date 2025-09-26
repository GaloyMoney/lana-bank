use thiserror::Error;

#[derive(Error, Debug)]
pub enum PaymentAllocationError {
    #[error("PaymentAllocationError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PaymentAllocationError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("PaymentAllocationError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("PaymentAllocationError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
}

es_entity::from_es_entity_error!(PaymentAllocationError);
