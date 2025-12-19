use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::primitives::WithdrawalId;

#[derive(Error, Debug)]
pub enum WithdrawalError {
    #[error("WithdrawalError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("WithdrawalError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("WithdrawalError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("WithdrawalError - DepositLedgerError: {0}")]
    DepositLedgerError(#[from] crate::ledger::error::DepositLedgerError),
    #[error("WithdrawalError - AlreadyConfirmed: {0}")]
    AlreadyConfirmed(WithdrawalId),
    #[error("WithdrawalError - AlreadyCancelled: {0}")]
    AlreadyCancelled(WithdrawalId),
    #[error("WithdrawalError - NotApproved: {0}")]
    NotApproved(WithdrawalId),
    #[error("WithdrawalError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("WithdrawalError - NotConfirmed: {0}")]
    NotConfirmed(WithdrawalId),
}

es_entity::from_es_entity_error!(WithdrawalError);

impl ErrorSeverity for WithdrawalError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::DepositLedgerError(_) => Level::ERROR,
            Self::AlreadyConfirmed(_) => Level::WARN,
            Self::AlreadyCancelled(_) => Level::WARN,
            Self::NotApproved(_) => Level::WARN,
            Self::AuditError(e) => e.severity(),
            Self::NotConfirmed(_) => Level::WARN,
        }
    }
}
