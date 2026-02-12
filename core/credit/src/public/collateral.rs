use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{
    collateral::Collateral,
    primitives::{
        CollateralDirection, CollateralId, CreditFacilityId, LedgerTxId, PendingCreditFacilityId,
        Satoshis,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct CollateralAdjustment {
    pub tx_id: LedgerTxId,
    pub abs_diff: Satoshis,
    pub direction: CollateralDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicCollateral {
    pub id: CollateralId,
    pub credit_facility_id: CreditFacilityId,
    pub pending_credit_facility_id: PendingCreditFacilityId,
    pub amount: Satoshis,
    pub adjustment: Option<CollateralAdjustment>,
}

impl From<&Collateral> for PublicCollateral {
    fn from(entity: &Collateral) -> Self {
        PublicCollateral {
            id: entity.id,
            credit_facility_id: entity.credit_facility_id,
            pending_credit_facility_id: entity.pending_credit_facility_id,
            amount: entity.amount,
            adjustment: entity.last_adjustment(),
        }
    }
}
