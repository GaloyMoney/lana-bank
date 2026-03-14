use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use money::UsdCents;

pub use crate::disbursal::DisbursalSettlement;
use crate::{
    disbursal::Disbursal,
    primitives::{CreditFacilityId, DisbursalId, DisbursalStatus},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicDisbursal {
    pub id: DisbursalId,
    pub credit_facility_id: CreditFacilityId,
    pub amount: UsdCents,
    pub status: DisbursalStatus,
    pub settlement: Option<DisbursalSettlement>,
}

impl From<&Disbursal> for PublicDisbursal {
    fn from(entity: &Disbursal) -> Self {
        PublicDisbursal {
            id: entity.id,
            credit_facility_id: entity.facility_id,
            amount: entity.amount,
            status: entity.status(),
            settlement: entity.settlement(),
        }
    }
}
