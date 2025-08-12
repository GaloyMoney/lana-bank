use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeycloakClientError {
    #[error("KeycloakClientError - Parse error: {0}")]
    ParseError(String),
    #[error("KeycloakClientError - Keycloak API error: {0}")]
    KeycloakError(#[from] keycloak::KeycloakError),
    #[error("KeycloakClientError - UUID parse error: {0}")]
    UuidError(#[from] uuid::Error),
    #[error("KeycloakClientError - User not found: {0}")]
    UserNotFound(String),
    #[error("KeycloakClientError - Multiple users found: {0}")]
    MultipleUsersFound(String),
}
