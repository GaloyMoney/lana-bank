use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CustomerSyncError {
    #[error("CustomerSyncError - JobError: {0}")]
    Job(#[from] ::job::error::JobError),
    #[error("CustomerSyncError - KeycloakClientError: {0}")]
    KeycloakClient(#[from] keycloak_client::KeycloakClientError),
    #[error("CustomerSyncError - RegisterEventHandler: {0}")]
    RegisterEventHandler(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl ErrorSeverity for CustomerSyncError {
    fn severity(&self) -> Level {
        match self {
            Self::Job(_) => Level::ERROR,
            Self::KeycloakClient(e) => e.severity(),
            Self::RegisterEventHandler(_) => Level::ERROR,
        }
    }
}
