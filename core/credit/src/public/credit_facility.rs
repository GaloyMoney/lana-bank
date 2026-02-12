use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use money::UsdCents;

use crate::{
    credit_facility::CreditFacility,
    primitives::{CreditFacilityId, LedgerTxId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicCreditFacility {
    pub id: CreditFacilityId,
    pub activation_tx_id: LedgerTxId,
    pub activated_at: DateTime<Utc>,
    pub amount: UsdCents,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<&CreditFacility> for PublicCreditFacility {
    fn from(entity: &CreditFacility) -> Self {
        PublicCreditFacility {
            id: entity.id,
            activation_tx_id: entity.activation_tx_id(),
            activated_at: entity.activated_at,
            amount: entity.amount,
            completed_at: entity.completed_at(),
        }
    }
}
