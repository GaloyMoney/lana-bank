use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CreditFacilityHistoryError {
    #[error("CreditFacilityHistoryError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CreditFacilityHistoryError - NewJobError: {0}")]
    NewJobError(#[from] job_new::error::JobError),
    #[error("PendingCreditFacilityError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
}

impl ErrorSeverity for CreditFacilityHistoryError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::NewJobError(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
        }
    }
}
