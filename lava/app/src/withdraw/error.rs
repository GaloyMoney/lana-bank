use thiserror::Error;

use crate::primitives::{UsdCents, WithdrawId};

#[derive(Error, Debug)]
pub enum WithdrawError {
    #[error("WithdrawError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("WithdrawError - LedgerError: {0}")]
    LedgerError(#[from] crate::ledger::error::LedgerError),
    #[error("WithdrawError - UserError: {0}")]
    CustomerError(#[from] crate::customer::error::CustomerError),
    #[error("WithdrawError - AlreadyConfirmed: {0}")]
    AlreadyConfirmed(WithdrawId),
    #[error("WithdrawError - AlreadyCancelled: {0}")]
    AlreadyCancelled(WithdrawId),
    #[error("WithdrawError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
    #[error("WithdrawError - InsufficientBalance: {0} < {1}")]
    InsufficientBalance(UsdCents, UsdCents),
    #[error("WithdrawError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
    #[error("WithdrawError - NotFound")]
    NotFound,
}

impl From<es_entity::EsEntityError> for WithdrawError {
    fn from(e: es_entity::EsEntityError) -> Self {
        match e {
            es_entity::EsEntityError::NotFound => WithdrawError::NotFound,
            es_entity::EsEntityError::UninitializedFieldError(e) => {
                panic!(
                    "Inconsistent data when initializing a Withdraw entity: {:?}",
                    e
                )
            }
        }
    }
}
