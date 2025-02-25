use thiserror::Error;

#[derive(Error, Debug)]
pub enum StatementLedgerError {
    #[error("StatementLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}
