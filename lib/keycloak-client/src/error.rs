use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum KeycloakClientError {
    #[error("KeycloakClientError - Parse error: {0}")]
    ParseError(String),
    #[error("KeycloakClientError - Keycloak API error: {0}")]
    ApiError(#[from] keycloak::KeycloakError),
    #[error("KeycloakClientError - UUID parse error: {0}")]
    UuidError(#[from] uuid::Error),
    #[error("KeycloakClientError - User not found: {0}")]
    UserNotFound(String),
    #[error("KeycloakClientError - Multiple users found: {0}")]
    MultipleUsersFound(String),
}

impl ErrorSeverity for KeycloakClientError {
    fn severity(&self) -> Level {
        match self {
            Self::ParseError(_) => Level::ERROR,
            Self::ApiError(_) => Level::ERROR,
            Self::UuidError(_) => Level::ERROR,
            Self::UserNotFound(_) => Level::WARN,
            Self::MultipleUsersFound(_) => Level::WARN,
        }
    }
}
