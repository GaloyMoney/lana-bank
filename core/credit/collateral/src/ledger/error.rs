use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CollateralLedgerError {
    #[error("CollateralLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CollateralLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("CollateralLedgerError - TxTemplate: {0}")]
    CalaTxTemplate(#[from] cala_ledger::tx_template::error::TxTemplateError),
    #[error("CollateralLedgerError - CalaAccount: {0}")]
    CalaAccount(#[from] cala_ledger::account::error::AccountError),
    #[error("CollateralLedgerError - CalaAccountSet: {0}")]
    CalaAccountSet(#[from] cala_ledger::account_set::error::AccountSetError),
}

impl ErrorSeverity for CollateralLedgerError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::CalaLedger(_) => Level::ERROR,
            Self::CalaTxTemplate(_) => Level::ERROR,
            Self::CalaAccount(_) => Level::ERROR,
            Self::CalaAccountSet(_) => Level::ERROR,
        }
    }
}
