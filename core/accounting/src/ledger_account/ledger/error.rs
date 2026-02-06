use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum LedgerAccountLedgerError {
    #[error("LedgerAccountLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("LedgerAccountLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("LedgerAccountLedgerError - CalaEntryError: {0}")]
    CalaEntry(#[from] cala_ledger::entry::error::EntryError),
    #[error("LedgerAccountLedgerError - CalaBalanceError: {0}")]
    CalaBalance(#[from] cala_ledger::balance::error::BalanceError),
    #[error("LedgerAccountLedgerError - CalaAccountSetError: {0}")]
    CalaAccountSet(#[from] cala_ledger::account_set::error::AccountSetError),
    #[error("LedgerAccountLedgerError - CalaAccountError: {0}")]
    CalaAccount(#[from] cala_ledger::account::error::AccountError),
    #[error("LedgerAccountError - ParseCurrencyError: {0}")]
    ParseCurrencyError(#[from] cala_ledger::ParseCurrencyError),
    #[error("LedgerAccountLedgerError - JournalError: {0}")]
    JournalError(#[from] crate::journal_error::JournalError),
    #[error("LedgerAccountError - ConversionError: {0}")]
    ConversionError(#[from] money::ConversionError),
}

impl ErrorSeverity for LedgerAccountLedgerError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::CalaLedger(_) => Level::ERROR,
            Self::CalaEntry(_) => Level::ERROR,
            Self::CalaBalance(_) => Level::ERROR,
            Self::CalaAccountSet(_) => Level::ERROR,
            Self::CalaAccount(_) => Level::ERROR,
            Self::ParseCurrencyError(_) => Level::WARN,
            Self::JournalError(e) => e.severity(),
            Self::ConversionError(e) => e.severity(),
        }
    }
}
