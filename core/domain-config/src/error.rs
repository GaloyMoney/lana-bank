use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum DomainConfigError {
    #[error("DomainConfigError - Invalid State: {0}")]
    InvalidState(String),
    #[error("DomainConfigError - Invalid Type: {0}")]
    InvalidType(String),
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

impl From<rust_decimal::Error> for DomainConfigError {
    fn from(value: rust_decimal::Error) -> Self {
        Self::InvalidType(value.to_string())
    }
}

impl ErrorSeverity for DomainConfigError {
    fn severity(&self) -> Level {
        match self {
            Self::InvalidState(_) => Level::ERROR,
            Self::InvalidType(_) => Level::ERROR,
            Self::Serde(_) => Level::ERROR,
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
        }
    }
}
