use thiserror::Error;

#[derive(Error, Debug)]
pub enum BankError {
    #[error("BankError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("BankError - EntityError: {0}")]
    EntityError(#[from] crate::entity::EntityError),
    #[error("BankError - LedgerError: {0}")]
    LedgerError(#[from] crate::ledger::error::LedgerError),
}
