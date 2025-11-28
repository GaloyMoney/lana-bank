use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum DashboardError {
    #[error("DashboardError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DashboardError - JobError: {0}")]
    Job(#[from] ::job::error::JobError),
    #[error("DashboardError - Authorization: {0}")]
    Authorization(#[from] authz::error::AuthorizationError),
}

impl ErrorSeverity for DashboardError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Job(_) => Level::ERROR,
            Self::Authorization(e) => e.severity(),
        }
    }
}
