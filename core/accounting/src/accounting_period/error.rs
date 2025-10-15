use chrono::NaiveDate;
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
    #[error(
        "AccountingPeriodError - ClosingDateBeforePeriodStart: {closing_date} < {period_start}"
    )]
    ClosingDateBeforePeriodStart {
        closing_date: NaiveDate,
        period_start: NaiveDate,
    },
    #[error("AccountingPeriodError - ClosingDateBeforePeriodEnd: {closing_date} < {period_end}")]
    ClosingDateBeforePeriodEnd {
        closing_date: NaiveDate,
        period_end: NaiveDate,
    },
    #[error(
        "AccountingPeriodError - ClosingDateAfterGracePeriod: {closing_date} > {grace_period_end}"
    )]
    ClosingDateAfterGracePeriod {
        closing_date: NaiveDate,
        grace_period_end: NaiveDate,
    },
}

es_entity::from_es_entity_error!(AccountingPeriodError);
