use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CommandInboxError {
    #[error("CommandInboxError - InboxError: {0}")]
    InboxError(#[from] obix::inbox::InboxError),
    #[error("CommandInboxError - CustomerError: {0}")]
    CustomerError(#[from] core_customer::error::CustomerError),
    #[error("CommandInboxError - DepositError: {0}")]
    DepositError(#[from] core_deposit::error::CoreDepositError),
    #[error("CommandInboxError - DuplicateIdempotencyKey")]
    DuplicateIdempotencyKey,
    #[error("CommandInboxError - CustomerNotFoundAfterProcessing")]
    CustomerNotFoundAfterProcessing,
    #[error("CommandInboxError - DepositNotFoundAfterProcessing")]
    DepositNotFoundAfterProcessing,
    #[error("CommandInboxError - WithdrawalNotFoundAfterProcessing")]
    WithdrawalNotFoundAfterProcessing,
}

impl ErrorSeverity for CommandInboxError {
    fn severity(&self) -> Level {
        match self {
            Self::InboxError(_) => Level::ERROR,
            Self::CustomerError(e) => e.severity(),
            Self::DepositError(e) => e.severity(),
            Self::DuplicateIdempotencyKey => Level::WARN,
            Self::CustomerNotFoundAfterProcessing => Level::ERROR,
            Self::DepositNotFoundAfterProcessing => Level::ERROR,
            Self::WithdrawalNotFoundAfterProcessing => Level::ERROR,
        }
    }
}
