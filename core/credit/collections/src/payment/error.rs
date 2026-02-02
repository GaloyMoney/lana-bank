use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

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
    #[error("PaymentError - CollectionsLedgerError: {0}")]
    CollectionsLedgerError(#[from] crate::ledger::error::CollectionsLedgerError),
    #[error("PaymentError - ObligationError: {0}")]
    ObligationError(#[from] crate::obligation::error::ObligationError),
    #[error("PaymentError - PaymentAllocationError: {0}")]
    PaymentAllocationError(#[from] crate::payment_allocation::error::PaymentAllocationError),
}

impl ErrorSeverity for PaymentError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::CollectionsLedgerError(e) => e.severity(),
            Self::ObligationError(e) => e.severity(),
            Self::PaymentAllocationError(e) => e.severity(),
        }
    }
}

es_entity::from_es_entity_error!(PaymentError);
