use authz::AuthorizationError;
use thiserror::Error;

#[derive(Debug, Error)]
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
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<AuthorizationError> for DomainConfigurationError {
    fn from(err: AuthorizationError) -> Self {
        match err {
            AuthorizationError::Unauthorized(_) => DomainConfigurationError::Forbidden,
            _ => DomainConfigurationError::Internal(err.into()),
        }
    }
}

es_entity::from_es_entity_error!(DomainConfigurationError);
