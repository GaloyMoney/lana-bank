use authz::error::AuthorizationError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainConfigurationError {
    #[error("DomainConfigurationError - NotSet")]
    NotSet,
    #[error("DomainConfigurationError - Forbidden")]
    Forbidden,
    #[error("DomainConfigurationError - Invalid: {0}")]
    Invalid(String),
    #[error("DomainConfigurationError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DomainConfigurationError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("DomainConfigurationError - Internal: {0}")]
    Internal(#[from] anyhow::Error),
}

impl From<AuthorizationError> for DomainConfigurationError {
    fn from(err: AuthorizationError) -> Self {
        match err {
            AuthorizationError::NotAuthorized => DomainConfigurationError::Forbidden,
            other => DomainConfigurationError::Internal(other.into()),
        }
    }
}

es_entity::from_es_entity_error!(DomainConfigurationError);
