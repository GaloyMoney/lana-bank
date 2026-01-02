use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum LedgerTransactionError {
    #[error("LedgerTransactionError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("LedgerTransactionError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("LedgerTransactionError - CalaEntryError: {0}")]
    CalaEntry(#[from] cala_ledger::entry::error::EntryError),
    #[error("LedgerTransactionError - CalaTransaction: {0}")]
    CalaTransaction(#[from] cala_ledger::transaction::error::TransactionError),
    #[error("LedgerTransactionError - CalaTxTemplate: {0}")]
    CalaTxTemplate(#[from] cala_ledger::tx_template::error::TxTemplateError),
    #[error("LedgerTransactionError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("LedgerTransactionError - JournalError: {0}")]
    JournalError(#[from] crate::journal::error::JournalError),
    #[error("LedgerTransactionError - Metadata: {0}")]
    MetadataError(#[from] serde_json::Error),
}

impl ErrorSeverity for LedgerTransactionError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::CalaLedger(_) => Level::ERROR,
            Self::CalaEntry(_) => Level::ERROR,
            Self::CalaTransaction(_) => Level::ERROR,
            Self::CalaTxTemplate(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::JournalError(e) => e.severity(),
            Self::MetadataError(_) => Level::ERROR,
        }
    }
}
