use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreDepositError {
    #[error("CoreDepositError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CoreDepositError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CoreDepositError - DepositAccountError: {0}")]
    DepositAccountError(#[from] crate::account::error::DepositAccountError),
    #[error("CoreDepositError - DepositError: {0}")]
    DepositError(#[from] crate::deposit::error::DepositError),
    #[error("CoreDepositError - WithdrawalError: {0}")]
    WithdrawalError(#[from] crate::withdrawal::error::WithdrawalError),
    #[error("CoreDepositError - DepositLedgerError: {0}")]
    DepositLedgerError(#[from] crate::ledger::error::DepositLedgerError),
}
