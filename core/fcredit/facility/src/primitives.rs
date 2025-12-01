use serde::{Deserialize, Serialize};

use core_money::UsdCents;
use credit_terms::balance_summary::CreditFacilityBalanceSummary;

es_entity::entity_id! {
    CreditFacilityProposalId,
    PendingCreditFacilityId,
    CreditFacilityId;

    CreditFacilityProposalId => PendingCreditFacilityId,

    CreditFacilityProposalId => CreditFacilityId,
    PendingCreditFacilityId => CreditFacilityId,

    CreditFacilityId => governance::ApprovalProcessId,
    CreditFacilityProposalId => governance::ApprovalProcessId,

    CreditFacilityId => job::JobId,

    CreditFacilityId => public_id::PublicIdTargetId,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct CreditFacilityReceivable {
    pub disbursed: UsdCents,
    pub interest: UsdCents,
}

impl From<CreditFacilityBalanceSummary> for CreditFacilityReceivable {
    fn from(balance: CreditFacilityBalanceSummary) -> Self {
        Self {
            disbursed: balance.disbursed_outstanding_payable(),
            interest: balance.interest_outstanding_payable(),
        }
    }
}
