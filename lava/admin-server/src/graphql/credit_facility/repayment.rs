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

impl From<lava_app::credit_facility::RepaymentStatus> for CreditFacilityRepaymentStatus {
    fn from(status: lava_app::credit_facility::RepaymentStatus) -> Self {
        match status {
            lava_app::credit_facility::RepaymentStatus::Paid => CreditFacilityRepaymentStatus::Paid,
            lava_app::credit_facility::RepaymentStatus::Due => CreditFacilityRepaymentStatus::Due,
            lava_app::credit_facility::RepaymentStatus::Overdue => {
                CreditFacilityRepaymentStatus::Overdue
            }
            lava_app::credit_facility::RepaymentStatus::Upcoming => {
                CreditFacilityRepaymentStatus::Upcoming
            }
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

impl From<lava_app::credit_facility::CreditFacilityRepaymentInPlan>
    for CreditFacilityRepaymentInPlan
{
    fn from(repayment: lava_app::credit_facility::CreditFacilityRepaymentInPlan) -> Self {
        match repayment {
            lava_app::credit_facility::CreditFacilityRepaymentInPlan::Disbursal(repayment) => {
                Self {
                    repayment_type: CreditFacilityRepaymentType::Disbursal,
                    status: repayment.status.into(),
                    initial: repayment.initial,
                    outstanding: repayment.outstanding,
                    accrual_at: repayment.accrual_at.into(),
                    due_at: repayment.due_at.into(),
                }
            }
            lava_app::credit_facility::CreditFacilityRepaymentInPlan::Interest(repayment) => Self {
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
