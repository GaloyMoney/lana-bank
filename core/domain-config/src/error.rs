use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::{primitives::DomainConfigKey, simple::SimpleType};

#[derive(Error, Debug)]
pub enum DomainConfigError {
    #[error("DomainConfigError - Invalid State: {0}")]
    InvalidState(String),
    #[error(
        "DomainConfigError - Invalid Simple Type for {key}: expected {expected}, found {found:?}"
    )]
    InvalidSimpleType {
        key: DomainConfigKey,
        expected: SimpleType,
        found: Option<SimpleType>,
    },
    #[error("DomainConfigError - Missing simple value for {0}")]
    MissingSimpleValue(DomainConfigKey),
    #[error("DomainConfigError - Invalid Simple Value: {0}")]
    InvalidSimpleValue(String),
    #[error("DomainConfigError - Config is simple for {key}: found simple type {found}")]
    InvalidConfigKind {
        key: DomainConfigKey,
        found: SimpleType,
    },
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
            Self::InvalidSimpleType { .. } => Level::ERROR,
            Self::MissingSimpleValue(_) => Level::ERROR,
            Self::InvalidSimpleValue(_) => Level::ERROR,
            Self::InvalidConfigKind { .. } => Level::ERROR,
            Self::Serde(_) => Level::ERROR,
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
        }
    }
}
