use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum LiquidationProcessError {
    #[error("LiquidationProcessError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("LiquidationProcessError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("LiquidationProcessError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("LiquidationProcessError - AlreadySatifissed")]
    AlreadySatisfied,
}

es_entity::from_es_entity_error!(LiquidationProcessError);

impl ErrorSeverity for LiquidationProcessError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(_) => Level::ERROR,
            Self::CursorDestructureError(_) => Level::ERROR,
        }
    }
}
