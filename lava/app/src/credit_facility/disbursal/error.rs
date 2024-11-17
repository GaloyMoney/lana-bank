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
    JobError(#[from] crate::job::error::JobError),
    #[error("DisbursalError - AlreadyConfirmed")]
    AlreadyConfirmed,
    #[error("DisbursalError - ApprovalInProgress")]
    ApprovalInProgress,
    #[error("DisbursalError - Denied")]
    Denied,
}

es_entity::from_es_entity_error!(DisbursalError);
