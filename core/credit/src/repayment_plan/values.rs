use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::primitives::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ObligationInPlan {
    pub obligation_type: ObligationType,
    pub status: RepaymentStatus,

    pub initial: UsdCents,
    pub outstanding: UsdCents,

    pub due_at: DateTime<Utc>,
    pub overdue_at: Option<DateTime<Utc>>,
    pub defaulted_at: Option<DateTime<Utc>>,
    pub recorded_at: DateTime<Utc>,
}

impl From<&RecordedObligationInPlan> for ObligationInPlan {
    fn from(obligation: &RecordedObligationInPlan) -> Self {
        obligation.values
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(super) struct RecordedObligationInPlan {
    pub obligation_id: ObligationId,
    pub values: ObligationInPlan,
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
