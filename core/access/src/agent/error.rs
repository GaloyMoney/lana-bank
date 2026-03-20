use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::repo::{AgentCreateError, AgentFindError, AgentModifyError, AgentQueryError};

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("AgentError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("AgentError - Create: {0}")]
    Create(#[from] AgentCreateError),
    #[error("AgentError - Modify: {0}")]
    Modify(#[from] AgentModifyError),
    #[error("AgentError - Find: {0}")]
    Find(#[from] AgentFindError),
    #[error("AgentError - Query: {0}")]
    Query(#[from] AgentQueryError),
    #[error("AgentError - KeycloakError: {0}")]
    KeycloakError(#[from] keycloak_client::KeycloakClientError),
}

impl ErrorSeverity for AgentError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::KeycloakError(_) => Level::ERROR,
        }
    }
}
