use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{
    payment_allocation::PaymentAllocation,
    primitives::{BeneficiaryId, ObligationId, ObligationType, PaymentAllocationId, UsdCents},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicPaymentAllocation {
    pub id: PaymentAllocationId,
    pub obligation_id: ObligationId,
    pub obligation_type: ObligationType,
    pub beneficiary_id: BeneficiaryId,
    pub amount: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub effective: chrono::NaiveDate,
}

impl From<&PaymentAllocation> for PublicPaymentAllocation {
    fn from(entity: &PaymentAllocation) -> Self {
        PublicPaymentAllocation {
            id: entity.id,
            obligation_id: entity.obligation_id,
            obligation_type: entity.obligation_type,
            beneficiary_id: entity.beneficiary_id,
            amount: entity.amount,
            recorded_at: entity.created_at(),
            effective: entity.effective,
        }
    }
}
