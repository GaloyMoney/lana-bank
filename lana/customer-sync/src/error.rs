use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomerSyncError {
    #[error("CustomerSyncError - JobError: {0}")]
    Job(#[from] ::job::error::JobError),
    #[error("CustomerSyncError - KeycloakAdminError: {0}")]
    KeycloakAdmin(#[from] keycloak_admin::KeycloakAdminError),
}
