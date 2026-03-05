use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::credit_facility::repo::{
    InterestAccrualCycleCreateError, InterestAccrualCycleFindError,
    InterestAccrualCycleModifyError, InterestAccrualCycleQueryError,
};

#[derive(Error, Debug, strum::IntoStaticStr)]
pub enum InterestAccrualCycleError {
    #[error("InterestAccrualCycleError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("InterestAccrualCycleError - Create: {0}")]
    Create(#[from] InterestAccrualCycleCreateError),
    #[error("InterestAccrualCycleError - Modify: {0}")]
    Modify(#[from] InterestAccrualCycleModifyError),
    #[error("InterestAccrualCycleError - Find: {0}")]
    Find(#[from] InterestAccrualCycleFindError),
    #[error("InterestAccrualCycleError - Query: {0}")]
    Query(#[from] InterestAccrualCycleQueryError),
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
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::JobError(_) => Level::ERROR,
            Self::AccrualsAlreadyPosted => Level::WARN,
            Self::InterestPeriodStartDatePastAccrualCycleDate => Level::ERROR,
            Self::NoNextAccrualPeriod => Level::ERROR,
        }
    }
}
