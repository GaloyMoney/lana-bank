use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CoreCreditCollectionError {
    #[error("CoreCreditCollectionError - ObligationError: {0}")]
    ObligationError(#[from] crate::obligation::error::ObligationError),
    #[error("CoreCreditCollectionError - PaymentError: {0}")]
    PaymentError(#[from] crate::payment::error::PaymentError),
    #[error("CoreCreditCollectionError - PaymentAllocationError: {0}")]
    PaymentAllocationError(#[from] crate::payment_allocation::error::PaymentAllocationError),
    #[error("CoreCreditCollectionError - CollectionLedgerError: {0}")]
    CollectionLedgerError(#[from] crate::ledger::error::CollectionLedgerError),
}

impl ErrorSeverity for CoreCreditCollectionError {
    fn severity(&self) -> Level {
        match self {
            Self::ObligationError(e) => e.severity(),
            Self::PaymentError(e) => e.severity(),
            Self::PaymentAllocationError(e) => e.severity(),
            Self::CollectionLedgerError(e) => e.severity(),
        }
    }
}
