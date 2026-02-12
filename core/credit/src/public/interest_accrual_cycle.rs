use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use core_credit_terms::InterestPeriod;
use money::UsdCents;

use crate::{
    credit_facility::interest_accrual_cycle::InterestAccrualCycle,
    primitives::{CreditFacilityId, EffectiveDate, InterestAccrualCycleId, LedgerTxId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct AccrualPosting {
    pub tx_id: LedgerTxId,
    pub amount: UsdCents,
    pub effective: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicInterestAccrualCycle {
    pub id: InterestAccrualCycleId,
    pub credit_facility_id: CreditFacilityId,
    pub period: InterestPeriod,
    pub due_at: EffectiveDate,
    pub posting: Option<AccrualPosting>,
}

impl From<&InterestAccrualCycle> for PublicInterestAccrualCycle {
    fn from(entity: &InterestAccrualCycle) -> Self {
        PublicInterestAccrualCycle {
            id: entity.id,
            credit_facility_id: entity.credit_facility_id,
            period: entity.period,
            due_at: EffectiveDate::from(entity.period.end),
            posting: entity.posting(),
        }
    }
}
