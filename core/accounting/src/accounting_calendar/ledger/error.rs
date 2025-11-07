use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountingCalendarLedgerError {
    #[error("AccountingCalendarLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("AccountingCalendarLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
}
