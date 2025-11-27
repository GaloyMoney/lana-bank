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
    #[error("CustodianError - FromHex: {0}")]
    FromHex(#[from] hex::FromHexError),
    #[error("CustodianError - InvalidEncryptionKey")]
    InvalidEncryptionKey,
}

es_entity::from_es_entity_error!(CustodianError);

impl ErrorSeverity for CustodianError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(_) => Level::ERROR,
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::FromHex(_) => Level::ERROR,
            Self::InvalidEncryptionKey => Level::ERROR,
        }
    }
}
