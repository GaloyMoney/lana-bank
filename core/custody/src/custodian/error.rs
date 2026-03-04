use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

pub use super::repo::{
    CustodianCreateError, CustodianFindError, CustodianModifyError, CustodianQueryError,
};

#[derive(Error, Debug)]
pub enum CustodianError {
    #[error("CustodianError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CustodianError - Create: {0}")]
    Create(#[from] CustodianCreateError),
    #[error("CustodianError - Modify: {0}")]
    Modify(#[from] CustodianModifyError),
    #[error("CustodianError - Find: {0}")]
    Find(#[from] CustodianFindError),
    #[error("CustodianError - Query: {0}")]
    Query(#[from] CustodianQueryError),
    #[error("CustodianError - Encryption: {0}")]
    Encryption(#[from] encryption::EncryptionError),
    #[error("CustodianError - StaleEncryptionKey: value was rotated to a newer key")]
    StaleEncryptionKey,
    #[error("CustodianError - Serde: {0}")]
    Serde(#[from] serde_json::Error),
}

impl ErrorSeverity for CustodianError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::Encryption(e) => e.severity(),
            Self::StaleEncryptionKey => Level::ERROR,
            Self::Serde(_) => Level::ERROR,
        }
    }
}
