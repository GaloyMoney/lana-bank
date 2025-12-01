use authz::{ActionPermission, AllOrOne, action_description::*, map_action};

use crate::primitives::CreditFacilityProposalId;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCreditFacilityAction {
    CreditFacilityProposal(CreditFacilityProposalAction),
}

impl CoreCreditFacilityAction {
    pub const CREDIT_FACILITY_PROPOSAL_CREATE: Self =
        CoreCreditFacilityAction::CreditFacilityProposal(CreditFacilityProposalAction::Create);
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum CreditFacilityProposalAction {
    Create,
}

pub type CreditFacilityProposalAllOrOne = AllOrOne<CreditFacilityProposalId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCreditFacilityObject {
    CreditFacilityProposal(CreditFacilityProposalAllOrOne),
}

impl CoreCreditFacilityObject {
    pub fn all_credit_facility_proposals() -> Self {
        CoreCreditFacilityObject::CreditFacilityProposal(AllOrOne::All)
    }
}
