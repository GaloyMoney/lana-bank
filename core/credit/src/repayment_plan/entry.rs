use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::primitives::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct ObligationDataForRepaymentPlanEntry {
    pub id: Option<ObligationId>,
    pub status: RepaymentStatus,

    pub initial: UsdCents,
    pub outstanding: UsdCents,

    pub due_at: DateTime<Utc>,
    pub overdue_at: Option<DateTime<Utc>>,
    pub defaulted_at: Option<DateTime<Utc>>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum CreditFacilityRepaymentPlanEntry {
    Disbursal(ObligationDataForRepaymentPlanEntry),
    Interest(ObligationDataForRepaymentPlanEntry),
}

impl PartialOrd for CreditFacilityRepaymentPlanEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CreditFacilityRepaymentPlanEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let ord = {
            let self_due_at = match self {
                Self::Disbursal(o) | Self::Interest(o) => o.due_at,
            };
            let other_due_at = match other {
                Self::Disbursal(o) | Self::Interest(o) => o.due_at,
            };
            self_due_at.cmp(&other_due_at)
        };

        ord.then_with(|| match (self, other) {
            (
                CreditFacilityRepaymentPlanEntry::Interest(_),
                CreditFacilityRepaymentPlanEntry::Disbursal(_),
            ) => std::cmp::Ordering::Less,
            (
                CreditFacilityRepaymentPlanEntry::Disbursal(_),
                CreditFacilityRepaymentPlanEntry::Interest(_),
            ) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        })
    }
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
