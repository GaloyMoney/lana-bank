use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum DomainConfigError {
    #[error("DomainConfigError - Invalid State: {0}")]
    InvalidState(String),
    #[error("DomainConfigError - Not Configured")]
    NotConfigured,
    #[error("DomainConfigError - Serde: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("DomainConfigError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DomainConfigError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("DomainConfigError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
}

es_entity::from_es_entity_error!(DomainConfigError);

impl ErrorSeverity for DomainConfigError {
    fn severity(&self) -> Level {
        match self {
            Self::InvalidState(_) => Level::ERROR,
            Self::NotConfigured => Level::WARN,
            Self::Serde(_) => Level::ERROR,
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
        }
    }
}
