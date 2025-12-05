use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum PublicIdError {
    #[error("PublicIdError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PublicIdError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("PublicIdError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
}

es_entity::from_es_entity_error!(PublicIdError);

impl ErrorSeverity for PublicIdError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
        }
    }
}
