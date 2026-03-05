use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::primitives::WithdrawalId;

use super::repo::{
    WithdrawalCreateError, WithdrawalFindError, WithdrawalModifyError, WithdrawalQueryError,
};

#[derive(Error, Debug, strum::IntoStaticStr)]
pub enum WithdrawalError {
    #[error("WithdrawalError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("WithdrawalError - Create: {0}")]
    Create(#[from] WithdrawalCreateError),
    #[error("WithdrawalError - Modify: {0}")]
    Modify(#[from] WithdrawalModifyError),
    #[error("WithdrawalError - Find: {0}")]
    Find(#[from] WithdrawalFindError),
    #[error("WithdrawalError - Query: {0}")]
    Query(#[from] WithdrawalQueryError),
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

impl ErrorSeverity for WithdrawalError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::DepositLedgerError(_) => Level::ERROR,
            Self::AlreadyConfirmed(_) => Level::WARN,
            Self::AlreadyCancelled(_) => Level::WARN,
            Self::NotApproved(_) => Level::WARN,
            Self::AuditError(e) => e.severity(),
            Self::NotConfirmed(_) => Level::WARN,
        }
    }
}
