use chrono::{DateTime, Utc};
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::primitives::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ObligationDataForEntry {
    pub id: Option<ObligationId>,
    pub status: RepaymentStatus,

    pub initial: UsdCents,
    pub outstanding: UsdCents,

    pub due_at: DateTime<Utc>,
    pub overdue_at: Option<DateTime<Utc>>,
    pub defaulted_at: Option<DateTime<Utc>>,

    pub recorded_at: DateTime<Utc>,
    pub effective: chrono::NaiveDate,
}

impl ObligationDataForEntry {
    fn is_existing_obligation(&self) -> bool {
        self.status != RepaymentStatus::Upcoming
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CreditFacilityRepaymentPlanEntry {
    Disbursal(ObligationDataForEntry),
    Interest(ObligationDataForEntry),
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

impl CreditFacilityRepaymentPlanEntry {
    pub fn is_already_accrued(&self) -> bool {
        match self {
            CreditFacilityRepaymentPlanEntry::Disbursal(data)
            | CreditFacilityRepaymentPlanEntry::Interest(data) => data.is_existing_obligation(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
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
