use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserOnboardingError {
    #[error("UserOnboardingError - JobError: {0}")]
    Job(#[from] ::job::error::JobError),
    #[error("UserOnboardingError - KeycloakAdminError: {0}")]
    KeycloakAdmin(#[from] keycloak_admin::KeycloakAdminError),
}
