use thiserror::Error;

#[derive(Error, Debug)]
pub enum OwnersEquityError {
    #[error("UserError - LedgerError: {0}")]
    LedgerError(#[from] crate::ledger::error::LedgerError),
}
