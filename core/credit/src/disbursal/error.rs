use thiserror::Error;

#[derive(Error, Debug)]
pub enum DisbursalError {
    #[error("DisbursalError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DisbursalError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("DisbursalError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("DisbursalError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("DisbursalError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("DisbursalError - GovernanceError: {0}")]
    GovernanceError(#[from] governance::error::GovernanceError),
    #[error("DisbursalError - ObligationError: {0}")]
    ObligationError(#[from] crate::obligation::error::ObligationError),
}

es_entity::from_es_entity_error!(DisbursalError);
