use thiserror::Error;

#[derive(Error, Debug)]
pub enum PaymentError {
    #[error("PaymentError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PaymentError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("PaymentError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("PaymentError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("PaymentError ObligationError: {0}")]
    ObligationError(#[from] crate::obligation::error::ObligationError),
    #[error("PaymentError - ObligationInstallmentError: {0}")]
    ObligationInstallmentError(
        #[from] crate::obligation_installment::error::ObligationInstallmentError,
    ),
}

es_entity::from_es_entity_error!(PaymentError);
