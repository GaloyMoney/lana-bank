use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::primitives::*;

#[derive(Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct ObligationInRepaymentPlan {
    pub status: RepaymentStatus,

    pub initial: UsdCents,
    pub outstanding: UsdCents,

    pub due_at: DateTime<Utc>,
    pub overdue_at: Option<DateTime<Utc>>,
    pub defaulted_at: Option<DateTime<Utc>>,
    pub recorded_at: DateTime<Utc>,
}

impl From<RepaymentInPlan> for ObligationInRepaymentPlan {
    fn from(repayment: RepaymentInPlan) -> Self {
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

#[derive(Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum CreditFacilityRepaymentPlanEntry {
    Disbursal(ObligationInRepaymentPlan),
    Interest(ObligationInRepaymentPlan),
}

impl From<RepaymentInPlan> for CreditFacilityRepaymentPlanEntry {
    fn from(obligation: RepaymentInPlan) -> Self {
        match obligation.obligation_type {
            ObligationType::Disbursal => Self::Disbursal(obligation.into()),
            ObligationType::Interest => Self::Interest(obligation.into()),
        }
    }
}

impl From<&ObligationInPlan> for CreditFacilityRepaymentPlanEntry {
    fn from(obligation: &ObligationInPlan) -> Self {
        obligation.values.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepaymentInPlan {
    pub obligation_type: ObligationType,
    pub status: RepaymentStatus,

    pub initial: UsdCents,
    pub outstanding: UsdCents,

    pub due_at: DateTime<Utc>,
    pub overdue_at: Option<DateTime<Utc>>,
    pub defaulted_at: Option<DateTime<Utc>>,
    pub recorded_at: DateTime<Utc>,
}

impl From<&ObligationInPlan> for RepaymentInPlan {
    fn from(obligation: &ObligationInPlan) -> Self {
        obligation.values
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(super) struct ObligationInPlan {
    pub obligation_id: ObligationId,
    pub values: RepaymentInPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepaymentStatus {
    Upcoming,
    NotYetDue,
    Due,
    Overdue,
    Defaulted,
    Paid,
}

impl From<ObligationStatus> for RepaymentStatus {
    fn from(status: ObligationStatus) -> Self {
        match status {
            ObligationStatus::NotYetDue => RepaymentStatus::NotYetDue,
            ObligationStatus::Due => RepaymentStatus::Due,
            ObligationStatus::Overdue => RepaymentStatus::Overdue,
            ObligationStatus::Defaulted => RepaymentStatus::Defaulted,
            ObligationStatus::Paid => RepaymentStatus::Paid,
        }
    }
}
