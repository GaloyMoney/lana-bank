use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

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
}

impl ErrorSeverity for ChartLedgerError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::CalaLedger(_) => Level::ERROR,
            Self::CalaAccountSet(_) => Level::ERROR,
            Self::Velocity(_) => Level::ERROR,
        }
    }
}
