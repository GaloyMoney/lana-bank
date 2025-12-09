use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(thiserror::Error, Debug)]
pub enum WalletError {
    #[error("WalletError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("WalletError - OutboxError: {0}")]
    OutboxError(#[from] outbox::error::OutboxError),
    #[error("WalletError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("WalletError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
}

es_entity::from_es_entity_error!(WalletError);

impl ErrorSeverity for WalletError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::OutboxError(e) => e.severity(),
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
        }
    }
}
