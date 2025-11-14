use thiserror::Error;

#[derive(Error, Debug)]
pub enum FiscalYearLedgerError {
    #[error("FiscalYearLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("FiscalYearLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("FiscalYearLedgerError - CalaAccountSet: {0}")]
    CalaAccountSet(#[from] cala_ledger::account_set::error::AccountSetError),
    #[error("ChartLedgerError - Velocity: {0}")]
    Velocity(#[from] cala_ledger::velocity::error::VelocityError),
}
