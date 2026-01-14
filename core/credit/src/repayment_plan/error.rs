use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CreditFacilityRepaymentPlanError {
    #[error("CreditFacilityRepaymentPlanError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CreditFacilityRepaymentPlanError - Job: {0}")]
    Job(#[from] job::error::JobError),
}

impl ErrorSeverity for CreditFacilityRepaymentPlanError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Job(_) => Level::ERROR,
        }
    }
}
