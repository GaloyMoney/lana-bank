use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreditFacilityProposalError {
    #[error("CreditFacilityProposalError - ApprovalProcessNotStarted")]
    ApprovalProcessNotStarted,
}
