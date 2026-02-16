use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use core_credit_terms::TermValues;
use money::UsdCents;

use crate::{
    pending_credit_facility::PendingCreditFacility,
    primitives::{
        CustomerId, PendingCreditFacilityCollateralizationState, PendingCreditFacilityId,
        PendingCreditFacilityStatus,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicPendingCreditFacility {
    pub id: PendingCreditFacilityId,
    pub status: PendingCreditFacilityStatus,
    pub collateralization_state: PendingCreditFacilityCollateralizationState,
    pub amount: UsdCents,
    pub terms: TermValues,
    pub customer_id: CustomerId,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<&PendingCreditFacility> for PublicPendingCreditFacility {
    fn from(entity: &PendingCreditFacility) -> Self {
        PublicPendingCreditFacility {
            id: entity.id,
            status: entity.status(),
            collateralization_state: entity.last_collateralization_state(),
            amount: entity.amount,
            terms: entity.terms,
            customer_id: entity.customer_id,
            created_at: entity.created_at(),
            completed_at: entity.completed_at(),
        }
    }
}
