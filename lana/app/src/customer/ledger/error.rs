use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomerLedgerError {
    #[error("CustomerLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CustomerLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("CustomerLedgerError - CalaAccountError: {0}")]
    CalaAccount(#[from] cala_ledger::account::error::AccountError),
}
