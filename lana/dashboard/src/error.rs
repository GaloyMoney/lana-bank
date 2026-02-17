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
    #[error("DashboardError - RegisterEventHandler: {0}")]
    RegisterEventHandler(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl ErrorSeverity for DashboardError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Job(_) => Level::ERROR,
            Self::Authorization(e) => e.severity(),
            Self::RegisterEventHandler(_) => Level::ERROR,
        }
    }
}
