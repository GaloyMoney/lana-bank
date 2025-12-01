use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CreditFacilityRepaymentPlanError {
    #[error("CreditFacilityRepaymentPlanError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl ErrorSeverity for CreditFacilityRepaymentPlanError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
        }
    }
}
