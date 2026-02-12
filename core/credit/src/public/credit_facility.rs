use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use money::{Satoshis, UsdCents};

use crate::{
    credit_facility::CreditFacility,
    primitives::{
        CollateralId, CreditFacilityId, CustomerId, LedgerTxId, LiquidationId, PriceOfOneBTC,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct LiquidationTrigger {
    pub liquidation_id: LiquidationId,
    pub trigger_price: PriceOfOneBTC,
    pub initially_expected_to_receive: UsdCents,
    pub initially_estimated_to_liquidate: Satoshis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicCreditFacility {
    pub id: CreditFacilityId,
    pub customer_id: CustomerId,
    pub collateral_id: CollateralId,
    pub activation_tx_id: LedgerTxId,
    pub activated_at: DateTime<Utc>,
    pub amount: UsdCents,
    pub completed_at: Option<DateTime<Utc>>,
    pub liquidation_trigger: Option<LiquidationTrigger>,
}

impl From<&CreditFacility> for PublicCreditFacility {
    fn from(entity: &CreditFacility) -> Self {
        PublicCreditFacility {
            id: entity.id,
            customer_id: entity.customer_id,
            collateral_id: entity.collateral_id,
            activation_tx_id: entity.activation_tx_id(),
            activated_at: entity.activated_at,
            amount: entity.amount,
            completed_at: entity.completed_at(),
            liquidation_trigger: entity.last_liquidation_trigger(),
        }
    }
}
