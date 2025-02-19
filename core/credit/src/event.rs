use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use core_money::{Satoshis, UsdCents};

use super::primitives::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreCreditEvent {
    FacilityCreated {
        id: CreditFacilityId,
        created_at: DateTime<Utc>,
    },
    FacilityApproved {
        id: CreditFacilityId,
    },
    FacilityActivated {
        id: CreditFacilityId,
        activated_at: DateTime<Utc>,
    },
    FacilityCompleted {
        id: CreditFacilityId,
        completed_at: DateTime<Utc>,
    },
    DisbursalExecuted {
        id: CreditFacilityId,
        amount: UsdCents,
        recorded_at: DateTime<Utc>,
    },
    FacilityRepaymentRecorded {
        id: CreditFacilityId,
        disbursal_amount: UsdCents,
        interest_amount: UsdCents,
        recorded_at: DateTime<Utc>,
    },
    FacilityCollateralUpdated {
        id: CreditFacilityId,
        new_amount: Satoshis,
        abs_diff: Satoshis,
        action: FacilityCollateralUpdateAction,
        recorded_at: DateTime<Utc>,
    },
    AccrualExecuted {
        id: CreditFacilityId,
        amount: UsdCents,
        accrued_at: DateTime<Utc>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FacilityCollateralUpdateAction {
    Add,
    Remove,
}
