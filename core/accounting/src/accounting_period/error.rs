use crate::primitives::ChartId;
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
    #[error(
        "AccountingPeriodError - ClosingDateOutOfGracePeriod: {closing_date} out of {grace_period_start} -> {grace_period_end}"
    )]
    ClosingDateOutOfGracePeriod {
        closing_date: NaiveDate,
        grace_period_start: NaiveDate,
        grace_period_end: NaiveDate,
    },
    #[error("AccountingPeriodError - ClosingMetadataNotFound")]
    ClosingMetadataNotFound { chart_id: ChartId },
    #[error("ChartOfAccountsError - CalaAccountSetError: {0}")]
    CalaAccountSet(#[from] cala_ledger::account_set::error::AccountSetError),
}

es_entity::from_es_entity_error!(AccountingPeriodError);
