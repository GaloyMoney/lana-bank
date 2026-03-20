use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum FxLedgerError {
    #[error("FxLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("FxLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("FxLedgerError - CalaAccountError: {0}")]
    CalaAccount(#[from] cala_ledger::account::error::AccountError),
    #[error("FxLedgerError - CalaAccountSetError: {0}")]
    AccountSetError(#[from] cala_ledger::account_set::error::AccountSetError),
    #[error("FxLedgerError - CalaTxTemplateError: {0}")]
    CalaTxTemplate(#[from] cala_ledger::tx_template::error::TxTemplateError),
    #[error("FxLedgerError - CalaTransactionError: {0}")]
    CalaTransaction(#[from] cala_ledger::transaction::error::TransactionError),
    #[error("FxLedgerError - FxPositionError: {0}")]
    FxPositionError(#[from] crate::position::error::FxPositionError),
    #[error("FxLedgerError - BtcNotAllowed")]
    BtcNotAllowed,
}

impl ErrorSeverity for FxLedgerError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::CalaLedger(_) => Level::ERROR,
            Self::CalaAccount(_) => Level::ERROR,
            Self::AccountSetError(_) => Level::ERROR,
            Self::CalaTxTemplate(_) => Level::ERROR,
            Self::CalaTransaction(_) => Level::ERROR,
            Self::FxPositionError(e) => e.severity(),
            Self::BtcNotAllowed => Level::WARN,
        }
    }
}
