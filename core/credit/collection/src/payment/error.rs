use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::repo::{PaymentCreateError, PaymentFindError, PaymentModifyError, PaymentQueryError};

#[derive(Error, Debug, strum::IntoStaticStr)]
pub enum PaymentError {
    #[error("PaymentError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PaymentError - Create: {0}")]
    Create(#[from] PaymentCreateError),
    #[error("PaymentError - Modify: {0}")]
    Modify(#[from] PaymentModifyError),
    #[error("PaymentError - Find: {0}")]
    Find(#[from] PaymentFindError),
    #[error("PaymentError - Query: {0}")]
    Query(#[from] PaymentQueryError),
    #[error("PaymentError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("PaymentError - CollectionLedgerError: {0}")]
    CollectionLedgerError(#[from] crate::ledger::error::CollectionLedgerError),
    #[error("PaymentError - ObligationError: {0}")]
    ObligationError(#[from] crate::obligation::error::ObligationError),
    #[error("PaymentError - PaymentAllocationError: {0}")]
    PaymentAllocationError(#[from] crate::payment_allocation::error::PaymentAllocationError),
}

impl ErrorSeverity for PaymentError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::CollectionLedgerError(e) => e.severity(),
            Self::ObligationError(e) => e.severity(),
            Self::PaymentAllocationError(e) => e.severity(),
        }
    }
}
