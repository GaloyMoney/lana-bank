use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

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

impl ErrorSeverity for PaymentAllocationError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
        }
    }
}

es_entity::from_es_entity_error!(PaymentAllocationError);
