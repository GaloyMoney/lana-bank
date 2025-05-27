use async_graphql::*;

use crate::primitives::*;

#[derive(async_graphql::Enum, Clone, Copy, PartialEq, Eq)]
pub enum CreditFacilityRepaymentType {
    Disbursal,
    Interest,
}

#[derive(async_graphql::Enum, Clone, Copy, PartialEq, Eq)]
pub enum CreditFacilityRepaymentStatus {
    Upcoming,
    NotYetDue,
    Due,
    Overdue,
    InRecovery,
    Defaulted,
    Paid,
}

impl From<lana_app::credit::RepaymentStatus> for CreditFacilityRepaymentStatus {
    fn from(status: lana_app::credit::RepaymentStatus) -> Self {
        match status {
            lana_app::credit::RepaymentStatus::Paid => CreditFacilityRepaymentStatus::Paid,
            lana_app::credit::RepaymentStatus::NotYetDue => CreditFacilityRepaymentStatus::Due,
            lana_app::credit::RepaymentStatus::Due => CreditFacilityRepaymentStatus::Due,
            lana_app::credit::RepaymentStatus::Overdue => CreditFacilityRepaymentStatus::Overdue,
            lana_app::credit::RepaymentStatus::InRecovery => {
                CreditFacilityRepaymentStatus::InRecovery
            }
            lana_app::credit::RepaymentStatus::Defaulted => {
                CreditFacilityRepaymentStatus::Defaulted
            }
            lana_app::credit::RepaymentStatus::Upcoming => CreditFacilityRepaymentStatus::Upcoming,
        }
    }
}

#[derive(SimpleObject)]
pub struct CreditFacilityRepaymentPlanEntry {
    pub repayment_type: CreditFacilityRepaymentType,
    pub status: CreditFacilityRepaymentStatus,
    pub initial: UsdCents,
    pub outstanding: UsdCents,
    pub accrual_at: Timestamp,
    pub due_at: Timestamp,
}

impl From<lana_app::credit::CreditFacilityRepaymentPlanEntry> for CreditFacilityRepaymentPlanEntry {
    fn from(repayment: lana_app::credit::CreditFacilityRepaymentPlanEntry) -> Self {
        match repayment {
            lana_app::credit::CreditFacilityRepaymentPlanEntry::Disbursal(repayment) => Self {
                repayment_type: CreditFacilityRepaymentType::Disbursal,
                status: repayment.status.into(),
                initial: repayment.initial,
                outstanding: repayment.outstanding,
                accrual_at: repayment.recorded_at.into(),
                due_at: repayment.due_at.into(),
            },
            lana_app::credit::CreditFacilityRepaymentPlanEntry::Interest(repayment) => Self {
                repayment_type: CreditFacilityRepaymentType::Interest,
                status: repayment.status.into(),
                initial: repayment.initial,
                outstanding: repayment.outstanding,
                accrual_at: repayment.recorded_at.into(),
                due_at: repayment.due_at.into(),
            },
        }
    }
}
