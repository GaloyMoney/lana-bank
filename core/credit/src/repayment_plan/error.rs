use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CreditFacilityRepaymentPlanError {
    #[error("CreditFacilityRepaymentPlanError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CreditFacilityRepaymentPlanError - Job: {0}")]
    Job(#[from] job::error::JobError),
    #[error("CreditFacilityRepaymentPlanError - RegisterEventHandler: {0}")]
    RegisterEventHandler(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("CoreCreditError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
}

impl ErrorSeverity for CreditFacilityRepaymentPlanError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Job(_) => Level::ERROR,
            Self::RegisterEventHandler(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
        }
    }
}
