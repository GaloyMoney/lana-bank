use thiserror::Error;

use crate::primitives::UserId;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("UserError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("UserError - EntityError: {0}")]
    EntityError(#[from] crate::entity::EntityError),
    #[error("UserError - LedgerError: {0}")]
    LedgerError(#[from] crate::ledger::error::LedgerError),
    #[error("UserError - CouldNotFindById: {0}")]
    CouldNotFindById(UserId),
    #[error("UserError - CouldNotFindEventByReference: {0}")]
    CouldNotEventFindByReference(String),
    #[error("UserError - UnexpectedCurrency")]
    UnexpectedCurrency,
}
