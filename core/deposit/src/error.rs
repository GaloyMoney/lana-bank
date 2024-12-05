use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreDepositError {
    #[error("CoreDepositError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CoreDepositError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CoreDepositError - CommitteeError: {0}")]
    DepositAccountError(#[from] crate::account::error::DepositAccountError),
}
