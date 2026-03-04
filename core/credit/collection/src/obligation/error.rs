use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::repo::{
    ObligationCreateError, ObligationFindError, ObligationModifyError, ObligationQueryError,
};

#[derive(Error, Debug)]
pub enum ObligationError {
    #[error("ObligationError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("ObligationError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("ObligationError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ObligationError - Create: {0}")]
    Create(#[from] ObligationCreateError),
    #[error("ObligationError - Modify: {0}")]
    Modify(#[from] ObligationModifyError),
    #[error("ObligationError - Find: {0}")]
    Find(#[from] ObligationFindError),
    #[error("ObligationError - Query: {0}")]
    Query(#[from] ObligationQueryError),
    #[error("CoreCreditError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("ObligationError - PaymentAllocationError: {0}")]
    PaymentAllocationError(#[from] crate::payment_allocation::error::PaymentAllocationError),
    #[error("ObligationError - CollectionLedgerError: {0}")]
    CollectionLedgerError(#[from] crate::ledger::error::CollectionLedgerError),
}

impl From<crate::payment_allocation::PaymentAllocationCreateError> for ObligationError {
    fn from(e: crate::payment_allocation::PaymentAllocationCreateError) -> Self {
        Self::PaymentAllocationError(e.into())
    }
}

impl From<crate::payment_allocation::PaymentAllocationFindError> for ObligationError {
    fn from(e: crate::payment_allocation::PaymentAllocationFindError) -> Self {
        Self::PaymentAllocationError(e.into())
    }
}

impl From<crate::payment_allocation::PaymentAllocationModifyError> for ObligationError {
    fn from(e: crate::payment_allocation::PaymentAllocationModifyError) -> Self {
        Self::PaymentAllocationError(e.into())
    }
}

impl From<crate::payment_allocation::PaymentAllocationQueryError> for ObligationError {
    fn from(e: crate::payment_allocation::PaymentAllocationQueryError) -> Self {
        Self::PaymentAllocationError(e.into())
    }
}

impl ErrorSeverity for ObligationError {
    fn severity(&self) -> Level {
        match self {
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::JobError(_) => Level::ERROR,
            Self::PaymentAllocationError(e) => e.severity(),
            Self::CollectionLedgerError(e) => e.severity(),
        }
    }
}
