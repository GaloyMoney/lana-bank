use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChartLedgerError {
    #[error("ChartLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ChartLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("ChartLedgerError - CalaAccountSet: {0}")]
    CalaAccountSet(#[from] cala_ledger::account_set::error::AccountSetError),
    #[error("ChartLedgerError - Velocity: {0}")]
    Velocity(#[from] cala_ledger::velocity::error::VelocityError),
    #[error("ChartLedgerError - CalaAccount: {0}")]
    Account(#[from] cala_ledger::account::error::AccountError),
}
