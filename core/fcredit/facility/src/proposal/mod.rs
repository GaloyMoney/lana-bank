mod entity;
pub mod error;
mod repo;

pub use entity::{CreditFacilityProposal, CreditFacilityProposalEvent, NewCreditFacilityProposal};
pub use repo::CreditFacilityProposalRepo;
pub const APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS: governance::ApprovalProcessType =
    governance::ApprovalProcessType::new("credit-facility-proposal");
