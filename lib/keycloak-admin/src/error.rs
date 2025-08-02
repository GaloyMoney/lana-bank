use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeycloakAdminError {
    #[error("KeycloakAdminError - Parse error: {0}")]
    ParseError(String),
    #[error("KeycloakAdminError - Keycloak API error: {0}")]
    KeycloakError(#[from] keycloak::KeycloakError),
    #[error("KeycloakAdminError - UUID parse error: {0}")]
    UuidError(#[from] uuid::Error),
}
