use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::primitives::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum RepaymentType {
    Disbursal,
    Interest,
}

impl From<&ObligationType> for RepaymentType {
    fn from(value: &ObligationType) -> Self {
        match value {
            ObligationType::Disbursal => Self::Disbursal,
            ObligationType::Interest => Self::Interest,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub struct CreditFacilityRepaymentPlanEntry {
    pub repayment_type: RepaymentType,
    pub obligation_id: Option<ObligationId>,
    pub status: RepaymentStatus,

    pub initial: UsdCents,
    pub outstanding: UsdCents,

    pub due_at: EffectiveDate,
    pub overdue_at: Option<EffectiveDate>,
    pub defaulted_at: Option<EffectiveDate>,

    pub recorded_at: DateTime<Utc>,
    pub effective: chrono::NaiveDate,

    #[serde(default)]
    pub accrual_ledger_tx_id: Option<LedgerTxId>,
}

impl PartialOrd for CreditFacilityRepaymentPlanEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CreditFacilityRepaymentPlanEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.due_at.cmp(&other.due_at).then_with(|| {
            match (self.repayment_type, other.repayment_type) {
                (RepaymentType::Interest, RepaymentType::Disbursal) => std::cmp::Ordering::Less,
                (RepaymentType::Disbursal, RepaymentType::Interest) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            }
        })
    }
}

impl CreditFacilityRepaymentPlanEntry {
    pub fn is_not_upcoming(&self) -> bool {
        self.status != RepaymentStatus::Upcoming
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
