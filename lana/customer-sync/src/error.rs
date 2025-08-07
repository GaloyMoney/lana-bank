use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomerSyncError {
    #[error("CustomerSyncError - JobError: {0}")]
    Job(#[from] ::job::error::JobError),
    #[error("CustomerSyncError - KeycloakClientError: {0}")]
    KeycloakClient(#[from] keycloak_client::KeycloakClientError),
}
