use thiserror::Error;

#[derive(Error, Debug)]
pub enum BfxIntegrationError {
    #[error("BfxIntegrationError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("BfxIntegrationError - EntityError: {0}")]
    EntityError(#[from] crate::entity::EntityError),
    #[error("BfxIntegrationError - LedgerError: {0}")]
    LedgerError(#[from] crate::ledger::error::LedgerError),
}
