use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

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
    #[error("InterestAccrualCycleError - NoNextAccrualPeriod")]
    NoNextAccrualPeriod,
}

impl ErrorSeverity for InterestAccrualCycleError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::JobError(_) => Level::ERROR,
            Self::AccrualsAlreadyPosted => Level::WARN,
            Self::InterestPeriodStartDatePastAccrualCycleDate => Level::ERROR,
            Self::NoNextAccrualPeriod => Level::ERROR,
        }
    }
}

es_entity::from_es_entity_error!(InterestAccrualCycleError);
