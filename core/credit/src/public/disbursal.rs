use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use money::UsdCents;

use crate::{
    disbursal::Disbursal,
    primitives::{CreditFacilityId, DisbursalId, LedgerTxId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct DisbursalSettlement {
    pub tx_id: LedgerTxId,
    pub effective: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicDisbursal {
    pub id: DisbursalId,
    pub credit_facility_id: CreditFacilityId,
    pub amount: UsdCents,
    pub settlement: Option<DisbursalSettlement>,
}

impl From<&Disbursal> for PublicDisbursal {
    fn from(entity: &Disbursal) -> Self {
        PublicDisbursal {
            id: entity.id,
            credit_facility_id: entity.facility_id,
            amount: entity.amount,
            settlement: entity.settlement(),
        }
    }
}
