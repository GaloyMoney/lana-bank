use thiserror::Error;

#[derive(Error, Debug)]
pub enum LiquidationObligationError {
    #[error("LiquidationObligationError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("LiquidationObligationError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("LiquidationObligationError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("LiquidationObligationError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("LiquidationObligationError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
}

es_entity::from_es_entity_error!(LiquidationObligationError);
