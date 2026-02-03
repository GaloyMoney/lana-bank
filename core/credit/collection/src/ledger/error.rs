use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CollectionLedgerError {
    #[error("CollectionLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CollectionLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("CollectionLedgerError - CalaTxTemplateError: {0}")]
    CalaTxTemplate(#[from] cala_ledger::tx_template::error::TxTemplateError),
    #[error("CollectionLedgerError - CalaVelocityError: {0}")]
    CalaVelocity(#[from] cala_ledger::velocity::error::VelocityError),
    #[error("CollectionLedgerError - PaymentAmountGreaterThanOutstandingObligations")]
    PaymentAmountGreaterThanOutstandingObligations,
}

impl ErrorSeverity for CollectionLedgerError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::CalaLedger(_) => Level::ERROR,
            Self::CalaTxTemplate(_) => Level::ERROR,
            Self::CalaVelocity(_) => Level::ERROR,
            Self::PaymentAmountGreaterThanOutstandingObligations => Level::WARN,
        }
    }
}
