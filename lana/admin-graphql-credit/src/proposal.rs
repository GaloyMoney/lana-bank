use async_graphql::*;

use crate::primitives::*;

pub use admin_graphql_shared::credit::CreditFacilityProposalBase;
pub use lana_app::credit::CreditFacilityProposalsByCreatedAtCursor;

use super::terms::TermsInput;

#[derive(InputObject)]
pub struct CreditFacilityProposalCreateInput {
    pub customer_id: UUID,
    pub facility: UsdCents,
    pub terms: TermsInput,
    pub custodian_id: Option<UUID>,
}

#[derive(InputObject)]
pub struct CreditFacilityProposalCustomerApprovalConcludeInput {
    pub credit_facility_proposal_id: UUID,
    pub approved: bool,
}
