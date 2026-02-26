use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CustodianError {
    #[error("CustodianError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CustodianError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("CustodianError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("CustodianError - Encryption: {0}")]
    Encryption(#[from] encryption::EncryptionError),
    #[error("CustodianError - StaleEncryptionKey: value was rotated to a newer key")]
    StaleEncryptionKey,
    #[error("CustodianError - Serde: {0}")]
    Serde(#[from] serde_json::Error),
}

es_entity::from_es_entity_error!(CustodianError);

impl ErrorSeverity for CustodianError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::Encryption(e) => e.severity(),
            Self::StaleEncryptionKey => Level::ERROR,
            Self::Serde(_) => Level::ERROR,
        }
    }
}
