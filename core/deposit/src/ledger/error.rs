use thiserror::Error;

#[derive(Error, Debug)]
pub enum DepositLedgerError {
    #[error("DepositLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DepositLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("DepositLedgerError - CalaLedger: {0}")]
    CalaAccount(#[from] cala_ledger::account::error::AccountError),
}
