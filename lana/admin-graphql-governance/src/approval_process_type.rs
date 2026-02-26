#[derive(async_graphql::Enum, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum ApprovalProcessType {
    WithdrawalApproval,
    DisbursalApproval,
    CreditFacilityProposalApproval,
}

pub use lana_app::governance::ApprovalProcessType as DomainApprovalProcessType;

impl From<&DomainApprovalProcessType> for ApprovalProcessType {
    fn from(process_type: &DomainApprovalProcessType) -> Self {
        if process_type == &lana_app::governance::APPROVE_WITHDRAWAL_PROCESS {
            Self::WithdrawalApproval
        } else if process_type == &lana_app::governance::APPROVE_DISBURSAL_PROCESS {
            Self::DisbursalApproval
        } else if process_type == &lana_app::governance::APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS {
            Self::CreditFacilityProposalApproval
        } else {
            panic!("Unknown approval process type: {process_type:?}");
        }
    }
}
