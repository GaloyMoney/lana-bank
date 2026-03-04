use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::entity::NewDomainConfigBuilderError;

use super::repo::{
    DomainConfigCreateError, DomainConfigFindError, DomainConfigModifyError, DomainConfigQueryError,
};

#[derive(Error, Debug)]
pub enum DomainConfigError {
    #[error("DomainConfigError - Invalid Key: {0}")]
    InvalidKey(String),
    #[error("DomainConfigError - Invalid State: {0}")]
    InvalidState(String),
    #[error("DomainConfigError - Invalid Type: {0}")]
    InvalidType(String),
    #[error("DomainConfigError - DuplicateKey")]
    DuplicateKey,
    #[error("DomainConfigError - Encryption")]
    Encryption(#[from] encryption::EncryptionError),
    #[error("DomainConfigError - Not Encrypted")]
    NotEncrypted(String),
    #[error("DomainConfigError - StaleEncryptionKey: value was rotated to a newer key")]
    StaleEncryptionKey,
    #[error("DomainConfigError - Serde: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("DomainConfigError - Sqlx: {0}")]
    Sqlx(sqlx::Error),
    #[error("DomainConfigError - Create: {0}")]
    Create(DomainConfigCreateError),
    #[error("DomainConfigError - Modify: {0}")]
    Modify(#[from] DomainConfigModifyError),
    #[error("DomainConfigError - Find: {0}")]
    Find(#[from] DomainConfigFindError),
    #[error("DomainConfigError - Query: {0}")]
    Query(#[from] DomainConfigQueryError),
    #[error("DomainConfigError - NewDomainConfigBuilderError: {0}")]
    BuildError(#[from] NewDomainConfigBuilderError),
    #[error("DomainConfigError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("DomainConfigError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
}

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

impl From<DomainConfigCreateError> for DomainConfigError {
    fn from(error: DomainConfigCreateError) -> Self {
        if error.was_duplicate() {
            return Self::DuplicateKey;
        }
        Self::Create(error)
    }
}

impl ErrorSeverity for DomainConfigError {
    fn severity(&self) -> Level {
        match self {
            Self::InvalidKey(_) => Level::ERROR,
            Self::InvalidState(_) => Level::ERROR,
            Self::InvalidType(_) => Level::ERROR,
            Self::DuplicateKey => Level::DEBUG,
            Self::Encryption(e) => e.severity(),
            Self::NotEncrypted(_) => Level::ERROR,
            Self::StaleEncryptionKey => Level::ERROR,
            Self::Serde(_) => Level::ERROR,
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::BuildError(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
        }
    }
}
