use thiserror::Error;

#[derive(Error, Debug)]
pub enum DisbursementError {
    #[error("DisbursementError - UserCannotApproveTwice")]
    UserCannotApproveTwice,
    #[error("DisbursementError - AlreadyApproved")]
    AlreadyApproved,
}
