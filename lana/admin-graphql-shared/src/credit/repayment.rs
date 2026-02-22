use async_graphql::*;

use super::primitives::*;

#[derive(async_graphql::Enum, Clone, Copy, PartialEq, Eq)]
pub enum CreditFacilityRepaymentType {
    Disbursal,
    Interest,
}

impl From<lana_app::credit::RepaymentType> for CreditFacilityRepaymentType {
    fn from(repayment_type: lana_app::credit::RepaymentType) -> Self {
        match repayment_type {
            lana_app::credit::RepaymentType::Disbursal => Self::Disbursal,
            lana_app::credit::RepaymentType::Interest => Self::Interest,
        }
    }
}

#[derive(async_graphql::Enum, Clone, Copy, PartialEq, Eq)]
pub enum CreditFacilityRepaymentStatus {
    Upcoming,
    NotYetDue,
    Due,
    Overdue,
    Defaulted,
    Paid,
}

impl From<lana_app::credit::RepaymentStatus> for CreditFacilityRepaymentStatus {
    fn from(status: lana_app::credit::RepaymentStatus) -> Self {
        match status {
            lana_app::credit::RepaymentStatus::Paid => CreditFacilityRepaymentStatus::Paid,
            lana_app::credit::RepaymentStatus::NotYetDue => {
                CreditFacilityRepaymentStatus::NotYetDue
            }
            lana_app::credit::RepaymentStatus::Due => CreditFacilityRepaymentStatus::Due,
            lana_app::credit::RepaymentStatus::Overdue => CreditFacilityRepaymentStatus::Overdue,
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
        Self {
            repayment_type: repayment.repayment_type.into(),
            status: repayment.status.into(),
            initial: repayment.initial,
            outstanding: repayment.outstanding,
            accrual_at: repayment.recorded_at.into(),
            due_at: repayment.due_at.end_of_day().into(),
        }
    }
}
