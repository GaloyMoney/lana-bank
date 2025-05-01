use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::primitives::*;

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

impl PartialOrd for RepaymentInPlan {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RepaymentInPlan {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.due_at.cmp(&other.due_at)
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
