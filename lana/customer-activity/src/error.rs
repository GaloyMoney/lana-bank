use core_customer::error::CustomerError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomerActivityError {
    #[error("CustomerActivityError - JobError: {0}")]
    Job(#[from] ::job::error::JobError),
    #[error("CustomerActivityError - Customer: {0}")]
    Customer(#[from] CustomerError),
}
