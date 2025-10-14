use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountingPeriodError {
    #[error("AccountingPeriodError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("AccountingPeriodError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("AccountingPeriodError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("AccountingPeriodError - NoOpenAccountingPeriodFound")]
    NoOpenAccountingPeriodFound,
    #[error("AccountingPeriodError - PeriodAlreadyClosed")]
    PeriodAlreadyClosed,
    #[error("AccountingPeriodError - CannotCalculatePeriodEnd")]
    CannotCalculatePeriodEnd,
}

es_entity::from_es_entity_error!(AccountingPeriodError);
