use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug, strum::IntoStaticStr)]
pub enum ManualTransactionLedgerError {
    #[error("ManualTransactionLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ManualTransactionLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("ManualTransactionLedgerError - CalaTxTemplate: {0}")]
    CalaTxTemplate(#[from] cala_ledger::tx_template::error::TxTemplateError),
}

impl ErrorSeverity for ManualTransactionLedgerError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::CalaLedger(cala_ledger::error::LedgerError::VelocityError(
                cala_ledger::velocity::error::VelocityError::Enforcement(_),
            )) => Level::WARN,
            Self::CalaLedger(_) => Level::ERROR,
            Self::CalaTxTemplate(_) => Level::ERROR,
        }
    }
}
