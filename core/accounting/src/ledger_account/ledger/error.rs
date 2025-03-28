use thiserror::Error;

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
    #[error("LedgerAccountError - ParseCurrencyError: {0}")]
    ParseCurrencyError(#[from] cala_ledger::ParseCurrencyError),
    #[error("LedgerAccountError - ConversionError: {0}")]
    ConversionError(#[from] core_money::ConversionError),
}
