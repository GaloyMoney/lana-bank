use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::primitives::*;

use super::values::*;

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ObligationDataForEntry {
    pub status: RepaymentStatus,

    pub initial: UsdCents,
    pub outstanding: UsdCents,

    pub due_at: DateTime<Utc>,
    pub overdue_at: Option<DateTime<Utc>>,
    pub defaulted_at: Option<DateTime<Utc>>,
    pub recorded_at: DateTime<Utc>,
}

impl From<ObligationInPlan> for ObligationDataForEntry {
    fn from(repayment: ObligationInPlan) -> Self {
        Self {
            status: repayment.status,
            initial: repayment.initial,
            outstanding: repayment.outstanding,
            due_at: repayment.due_at,
            overdue_at: repayment.overdue_at,
            defaulted_at: repayment.defaulted_at,
            recorded_at: repayment.recorded_at,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum CreditFacilityRepaymentPlanEntry {
    Disbursal(ObligationDataForEntry),
    Interest(ObligationDataForEntry),
}

impl From<ObligationInPlan> for CreditFacilityRepaymentPlanEntry {
    fn from(obligation: ObligationInPlan) -> Self {
        match obligation.obligation_type {
            ObligationType::Disbursal => Self::Disbursal(obligation.into()),
            ObligationType::Interest => Self::Interest(obligation.into()),
        }
    }
}

impl From<&RecordedObligationInPlan> for CreditFacilityRepaymentPlanEntry {
    fn from(obligation: &RecordedObligationInPlan) -> Self {
        obligation.values.into()
    }
}

impl PartialOrd for CreditFacilityRepaymentPlanEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CreditFacilityRepaymentPlanEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_due_at = match self {
            CreditFacilityRepaymentPlanEntry::Disbursal(o) => o.due_at,
            CreditFacilityRepaymentPlanEntry::Interest(o) => o.due_at,
        };

        let other_due_at = match other {
            CreditFacilityRepaymentPlanEntry::Disbursal(o) => o.due_at,
            CreditFacilityRepaymentPlanEntry::Interest(o) => o.due_at,
        };

        self_due_at.cmp(&other_due_at)
    }
}
