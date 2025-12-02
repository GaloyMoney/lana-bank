use thiserror::Error;

#[derive(Error, Debug)]
pub enum PendingCreditFacilityError {
    #[error("PendingCreditFacilityError - ApprovalInProgress")]
    ApprovalInProgress,
    #[error("PendingCreditFacilityError - BelowMarginLimit")]
    BelowMarginLimit,
}
