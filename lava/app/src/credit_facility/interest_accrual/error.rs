use thiserror::Error;

#[derive(Error, Debug)]
pub enum InterestAccrualError {
    #[error("InterestAccrualError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("InterestAccrualError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
    #[error("InterestAccrualError - AlreadyAccrued")]
    AlreadyAccrued,
    #[error("InterestAccrualError - InterestPeriodStartDatePastAccrualDate")]
    InterestPeriodStartDatePastAccrualDate,
    #[error("InterestAccrualError - NotFound")]
    NotFound,
}

impl From<es_entity::EsEntityError> for InterestAccrualError {
    fn from(e: es_entity::EsEntityError) -> Self {
        match e {
            es_entity::EsEntityError::NotFound => InterestAccrualError::NotFound,
            es_entity::EsEntityError::UninitializedFieldError(e) => {
                panic!(
                    "Inconsistent data when initializing a InterestAccrual entity: {:?}",
                    e
                )
            }
        }
    }
}
