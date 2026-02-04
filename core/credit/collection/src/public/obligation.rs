use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{
    obligation::Obligation,
    primitives::{BeneficiaryId, EffectiveDate, ObligationId, ObligationType, UsdCents},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicObligation {
    pub id: ObligationId,
    pub obligation_type: ObligationType,
    pub beneficiary_id: BeneficiaryId,
    pub initial_amount: UsdCents,
    pub outstanding_amount: UsdCents,
    pub due_at: EffectiveDate,
    pub overdue_at: Option<EffectiveDate>,
    pub defaulted_at: Option<EffectiveDate>,
    pub recorded_at: DateTime<Utc>,
    pub effective: chrono::NaiveDate,
}

impl From<&Obligation> for PublicObligation {
    fn from(entity: &Obligation) -> Self {
        let dates = entity.lifecycle_dates();
        PublicObligation {
            id: entity.id,
            obligation_type: entity.obligation_type,
            beneficiary_id: entity.beneficiary_id,
            initial_amount: entity.initial_amount,
            outstanding_amount: entity.outstanding(),
            due_at: dates.due,
            overdue_at: dates.overdue,
            defaulted_at: dates.defaulted,
            recorded_at: entity.created_at(),
            effective: entity.effective,
        }
    }
}
