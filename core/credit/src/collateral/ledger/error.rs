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
}

impl ErrorSeverity for CollateralLedgerError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::CalaLedger(_) => Level::ERROR,
            Self::CalaTxTemplate(_) => Level::ERROR,
        }
    }
}
