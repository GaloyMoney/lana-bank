use thiserror::Error;

#[derive(Error, Debug)]
pub enum DisbursementError {
    #[error("DisbursementError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DisbursementError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("DisbursementError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
    #[error("DisbursementError - AlreadyConfirmed")]
    AlreadyConfirmed,
    #[error("DisbursementError - ApprovalInProgress")]
    ApprovalInProgress,
    #[error("DisbursementError - Denied")]
    Denied,
}

es_entity::from_es_entity_error!(DisbursementError);
