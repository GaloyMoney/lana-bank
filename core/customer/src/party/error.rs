use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum PartyError {
    #[error("PartyError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PartyError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("PartyError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("PartyError - EmailAlreadyExists")]
    EmailAlreadyExists,
    #[error("PartyError - TelegramHandleAlreadyExists")]
    TelegramHandleAlreadyExists,
}

es_entity::from_es_entity_error!(PartyError);

impl ErrorSeverity for PartyError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::EmailAlreadyExists => Level::WARN,
            Self::TelegramHandleAlreadyExists => Level::WARN,
        }
    }
}
