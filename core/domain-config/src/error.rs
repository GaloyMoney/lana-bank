use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::entity::NewDomainConfigBuilderError;

#[derive(Error, Debug)]
pub enum DomainConfigError {
    #[error("DomainConfigError - Invalid Key: {0}")]
    InvalidKey(String),
    #[error("DomainConfigError - Invalid State: {0}")]
    InvalidState(String),
    #[error("DomainConfigError - Not Configured")]
    NotConfigured,
    #[error("DomainConfigError - No default value defined for config key {0}")]
    NoDefault(String),
    #[error("DomainConfigError - Invalid Type: {0}")]
    InvalidType(String),
    #[error("DomainConfigError - DuplicateKey")]
    DuplicateKey,
    #[error("DomainConfigError - Serde: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("DomainConfigError - Sqlx: {0}")]
    Sqlx(sqlx::Error),
    #[error("DomainConfigError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("DomainConfigError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("DomainConfigError - NewDomainConfigBuilderError: {0}")]
    BuildError(#[from] NewDomainConfigBuilderError),
}

es_entity::from_es_entity_error!(DomainConfigError);

impl From<sqlx::Error> for DomainConfigError {
    fn from(error: sqlx::Error) -> Self {
        if let Some(err) = error.as_database_error()
            && let Some(constraint) = err.constraint()
            && constraint.contains("core_domain_configs_key")
        {
            return Self::DuplicateKey;
        }
        Self::Sqlx(error)
    }
}

impl ErrorSeverity for DomainConfigError {
    fn severity(&self) -> Level {
        match self {
            Self::InvalidKey(_) => Level::ERROR,
            Self::InvalidState(_) => Level::ERROR,
            Self::NotConfigured => Level::WARN,
            Self::NoDefault(_) => Level::WARN,
            Self::InvalidType(_) => Level::ERROR,
            Self::DuplicateKey => Level::WARN,
            Self::Serde(_) => Level::ERROR,
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::BuildError(_) => Level::ERROR,
        }
    }
}
