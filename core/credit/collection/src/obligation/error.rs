use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum ObligationError {
    #[error("ObligationError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("ObligationError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("ObligationError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ObligationError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("ObligationError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("CoreCreditError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("ObligationError - PaymentAllocationError: {0}")]
    PaymentAllocationError(#[from] crate::payment_allocation::error::PaymentAllocationError),
    #[error("ObligationError - CollectionLedgerError: {0}")]
    CollectionLedgerError(#[from] crate::ledger::error::CollectionLedgerError),
}

impl ErrorSeverity for ObligationError {
    fn severity(&self) -> Level {
        match self {
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::JobError(_) => Level::ERROR,
            Self::PaymentAllocationError(e) => e.severity(),
            Self::CollectionLedgerError(e) => e.severity(),
        }
    }
}

es_entity::from_es_entity_error!(ObligationError);
