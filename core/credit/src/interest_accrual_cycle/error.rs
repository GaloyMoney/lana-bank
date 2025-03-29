use thiserror::Error;

#[derive(Error, Debug)]
pub enum InterestAccrualCycleError {
    #[error("InterestAccrualCycleError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("InterestAccrualCycleError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("InterestAccrualCycleError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("InterestAccrualCycleError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("InterestAccrualCycleError - AccrualsAlreadyPosted")]
    AccrualsAlreadyPosted,
    #[error("InterestAccrualCycleError - InterestPeriodStartDatePastAccrualCycleDate")]
    InterestPeriodStartDatePastAccrualCycleDate,
}

es_entity::from_es_entity_error!(InterestAccrualCycleError);
