use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{
    credit_facility_proposal::CreditFacilityProposal,
    primitives::{CreditFacilityProposalId, CreditFacilityProposalStatus, CustomerId},
};
use core_credit_terms::TermValues;
use money::UsdCents;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicCreditFacilityProposal {
    pub id: CreditFacilityProposalId,
    pub status: CreditFacilityProposalStatus,
    pub amount: UsdCents,
    pub terms: TermValues,
    pub customer_id: CustomerId,
    pub created_at: DateTime<Utc>,
}

impl From<&CreditFacilityProposal> for PublicCreditFacilityProposal {
    fn from(entity: &CreditFacilityProposal) -> Self {
        PublicCreditFacilityProposal {
            id: entity.id,
            status: entity.status(),
            amount: entity.amount,
            terms: entity.terms,
            customer_id: entity.customer_id,
            created_at: entity.created_at(),
        }
    }
}
