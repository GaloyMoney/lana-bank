use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeneralLedgerError {
    #[error("GeneralLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("GeneralLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("GeneralLedgerError - CalaEntryError: {0}")]
    CalaEntry(#[from] cala_ledger::entry::error::EntryError),
    #[error("GeneralLedgerError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("GeneralLedgerError - UnexpectedCurrency")]
    UnexpectedCurrency,
    #[error("GeneralLedgerError - ConversionError: {0}")]
    ConversionError(#[from] core_money::ConversionError),
    #[error("GeneralLedgerError - ParseCurrencyError: {0}")]
    ParseCurrencyError(#[from] cala_ledger::ParseCurrencyError),
}
