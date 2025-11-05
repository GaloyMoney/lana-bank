use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountingCalendarError {
    #[error("AccountingCalendarError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("AccountingCalendarError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("AccountingCalendarError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("AccountingCalendarError - AccountingCalendarLedgerError: {0}")]
    AccountingCalendarLedgerError(#[from] super::ledger::error::AccountingCalendarLedgerError),
    #[error("AccountingCalendarError - AccountPeriodStartNotFound")]
    AccountPeriodStartNotFound,
}

es_entity::from_es_entity_error!(AccountingCalendarError);
