use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{
    payment::Payment,
    primitives::{BeneficiaryId, PaymentId, UsdCents},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicPayment {
    pub id: PaymentId,
    pub beneficiary_id: BeneficiaryId,
    pub amount: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub effective: chrono::NaiveDate,
}

impl From<&Payment> for PublicPayment {
    fn from(entity: &Payment) -> Self {
        PublicPayment {
            id: entity.id,
            beneficiary_id: entity.beneficiary_id,
            amount: entity.amount,
            recorded_at: entity.created_at(),
            effective: entity.effective,
        }
    }
}
