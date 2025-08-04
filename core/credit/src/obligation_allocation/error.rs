use thiserror::Error;

#[derive(Error, Debug)]
pub enum ObligationAllocationError {
    #[error("ObligationAllocationError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ObligationAllocationError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("ObligationAllocationError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("ObligationAllocationError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
}

es_entity::from_es_entity_error!(ObligationAllocationError);
