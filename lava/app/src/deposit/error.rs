use thiserror::Error;

use crate::primitives::DepositId;

#[derive(Error, Debug)]
pub enum DepositError {
    #[error("DepositError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DepositError - LedgerError: {0}")]
    LedgerError(#[from] crate::ledger::error::LedgerError),
    #[error("DepositError - UserError: {0}")]
    CustomerError(#[from] crate::customer::error::CustomerError),
    #[error("DepositError - CouldNotFindById: {0}")]
    CouldNotFindById(DepositId),
    #[error("DepositError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
    #[error("DepositError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
    #[error("DepositError - NotFound")]
    NotFound,
}

impl From<es_entity::EsEntityError> for DepositError {
    fn from(e: es_entity::EsEntityError) -> Self {
        match e {
            es_entity::EsEntityError::NotFound => DepositError::NotFound,
            es_entity::EsEntityError::UninitializedFieldError(e) => {
                panic!(
                    "Inconsistent data when initializing a Deposit entity: {:?}",
                    e
                )
            }
        }
    }
}
