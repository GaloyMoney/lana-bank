use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::primitives::*;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreCreditCollectionEvent {
    PaymentReceived {
        payment_id: PaymentId,
        beneficiary_id: BeneficiaryId,
        amount: UsdCents,
        recorded_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
    },
    PaymentAllocated {
        beneficiary_id: BeneficiaryId,
        obligation_id: ObligationId,
        obligation_type: ObligationType,
        allocation_id: PaymentAllocationId,
        amount: UsdCents,
        recorded_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
    },
    ObligationCreated {
        id: ObligationId,
        obligation_type: ObligationType,
        beneficiary_id: BeneficiaryId,
        amount: UsdCents,
        due_at: EffectiveDate,
        overdue_at: Option<EffectiveDate>,
        defaulted_at: Option<EffectiveDate>,
        recorded_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
    },
    ObligationDue {
        id: ObligationId,
        beneficiary_id: BeneficiaryId,
        obligation_type: ObligationType,
        amount: UsdCents,
    },
    ObligationOverdue {
        id: ObligationId,
        beneficiary_id: BeneficiaryId,
        amount: UsdCents,
    },
    ObligationDefaulted {
        id: ObligationId,
        beneficiary_id: BeneficiaryId,
        amount: UsdCents,
    },
    ObligationCompleted {
        id: ObligationId,
        beneficiary_id: BeneficiaryId,
    },
}
