use thiserror::Error;

#[derive(Error, Debug)]
pub enum ObligationError {
    #[error("ObligationError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("ObligationError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ObligationError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("ObligationError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("ObligationError - InvalidStatusTransitionToOverdue")]
    InvalidStatusTransitionToOverdue,
    #[error("ObligationError - PaymentAmountGreaterThanOutstandingObligations")]
    PaymentAmountGreaterThanOutstandingObligations,
}

es_entity::from_es_entity_error!(ObligationError);
