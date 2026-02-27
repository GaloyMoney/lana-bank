use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum LiquidationError {
    #[error("LiquidationError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("LiquidationError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("LiquidationError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("LiquidationError - AlreadySatisfied")]
    AlreadySatisfied,
    #[error("LiquidationError - AlreadyCompleted")]
    AlreadyCompleted,
    #[error("LiquidationError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("LiquidationError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
}

es_entity::from_es_entity_error!(LiquidationError);

impl ErrorSeverity for LiquidationError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::AlreadySatisfied | Self::AlreadyCompleted => Level::WARN,
            Self::AuthorizationError(e) => e.severity(),
            Self::JobError(_) => Level::ERROR,
        }
    }
}
