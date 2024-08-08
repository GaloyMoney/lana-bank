use thiserror::Error;

use crate::primitives::DepositId;

#[derive(Error, Debug)]
pub enum DepositError {
    #[error("DepositError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DepositError - EntityError: {0}")]
    EntityError(#[from] crate::entity::EntityError),
    #[error("DepositError - LedgerError: {0}")]
    LedgerError(#[from] crate::ledger::error::LedgerError),
    #[error("DepositError - UserError: {0}")]
    CustomerError(#[from] crate::customer::error::CustomerError),
    #[error("DepositError - CouldNotFindById: {0}")]
    CouldNotFindById(DepositId),
    #[error("LoanError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
}
