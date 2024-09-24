use thiserror::Error;

use crate::primitives::CustomerId;

#[derive(Error, Debug)]
pub enum CreditFacilityError {
    #[error("CreditFacilityError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CreditFacilityError - EntityError: {0}")]
    EntityError(#[from] crate::entity::EntityError),
    #[error("CreditFacilityError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
    #[error("CreditFacilityError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
    #[error("CreditFacilityError - CustomerNotFound: {0}")]
    CustomerNotFound(CustomerId),
    #[error("CreditFacilityError- CustomerError: '{0}'")]
    CustomerError(#[from] crate::customer::error::CustomerError),
}
