use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum JournalError {
    #[error("JournalError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("JournalError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("JournalError - CalaEntryError: {0}")]
    CalaEntry(#[from] cala_ledger::entry::error::EntryError),
    #[error("JournalError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("JournalError - UnexpectedCurrency")]
    UnexpectedCurrency,
    #[error("JournalError - ConversionError: {0}")]
    ConversionError(#[from] money::ConversionError),
    #[error("JournalError - ParseCurrencyError: {0}")]
    ParseCurrencyError(#[from] cala_ledger::ParseCurrencyError),
}

impl ErrorSeverity for JournalError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::CalaLedger(_) => Level::ERROR,
            Self::CalaEntry(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::UnexpectedCurrency => Level::ERROR,
            Self::ConversionError(e) => e.severity(),
            Self::ParseCurrencyError(_) => Level::WARN,
        }
    }
}
