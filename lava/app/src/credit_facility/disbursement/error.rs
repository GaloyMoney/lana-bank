use thiserror::Error;

#[derive(Error, Debug)]
pub enum DisbursementError {
    #[error("DisbursementError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DisbursementError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
    #[error("DisbursementError - UserCannotApproveTwice")]
    UserCannotApproveTwice,
    #[error("DisbursementError - AlreadyApproved")]
    AlreadyApproved,
    #[error("DisbursementError - NotFound")]
    NotFound,
}

impl From<es_entity::EsEntityError> for DisbursementError {
    fn from(e: es_entity::EsEntityError) -> Self {
        match e {
            es_entity::EsEntityError::NotFound => DisbursementError::NotFound,
            es_entity::EsEntityError::UninitializedFieldError(e) => {
                panic!(
                    "Inconsistent data when initializing a Disbursement entity: {:?}",
                    e
                )
            }
        }
    }
}
