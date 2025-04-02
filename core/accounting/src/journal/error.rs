use thiserror::Error;

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
    ConversionError(#[from] core_money::ConversionError),
    #[error("JournalError - ParseCurrencyError: {0}")]
    ParseCurrencyError(#[from] cala_ledger::ParseCurrencyError),
}
