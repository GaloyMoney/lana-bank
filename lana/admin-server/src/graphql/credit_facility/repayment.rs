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
    Due,
    Overdue,
    Paid,
}

impl From<lana_app::credit::RepaymentStatus> for CreditFacilityRepaymentStatus {
    fn from(status: lana_app::credit::RepaymentStatus) -> Self {
        match status {
            lana_app::credit::RepaymentStatus::Paid => CreditFacilityRepaymentStatus::Paid,
            lana_app::credit::RepaymentStatus::Due => CreditFacilityRepaymentStatus::Due,
            lana_app::credit::RepaymentStatus::Overdue => CreditFacilityRepaymentStatus::Overdue,
            lana_app::credit::RepaymentStatus::Upcoming => CreditFacilityRepaymentStatus::Upcoming,
        }
    }
}

#[derive(SimpleObject)]
pub struct CreditFacilityRepaymentInPlan {
    pub repayment_type: CreditFacilityRepaymentType,
    pub status: CreditFacilityRepaymentStatus,
    pub initial: UsdCents,
    pub outstanding: UsdCents,
    pub accrual_at: Timestamp,
    pub due_at: Timestamp,
}

impl From<lana_app::credit::CreditFacilityRepaymentInPlan> for CreditFacilityRepaymentInPlan {
    fn from(repayment: lana_app::credit::CreditFacilityRepaymentInPlan) -> Self {
        match repayment {
            lana_app::credit::CreditFacilityRepaymentInPlan::Disbursal(repayment) => Self {
                repayment_type: CreditFacilityRepaymentType::Disbursal,
                status: repayment.status.into(),
                initial: repayment.initial,
                outstanding: repayment.outstanding,
                accrual_at: repayment.accrual_at.into(),
                due_at: repayment.due_at.into(),
            },
            lana_app::credit::CreditFacilityRepaymentInPlan::Interest(repayment) => Self {
                repayment_type: CreditFacilityRepaymentType::Interest,
                status: repayment.status.into(),
                initial: repayment.initial,
                outstanding: repayment.outstanding,
                accrual_at: repayment.accrual_at.into(),
                due_at: repayment.due_at.into(),
            },
        }
    }
}
