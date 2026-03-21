use async_graphql::*;

use crate::primitives::*;

#[derive(SimpleObject)]
pub(super) struct CreditFacilityBalance {
    facility_remaining: UsdCents,
    disbursed: Disbursed,
    interest: Interest,
    outstanding: UsdCents,
    outstanding_payable: UsdCents,
    due_outstanding: UsdCents,
    collateral: Satoshis,
    payments_unapplied: UsdCents,
}

impl From<lana_app::credit::CreditFacilityBalanceSummary> for CreditFacilityBalance {
    fn from(balance: lana_app::credit::CreditFacilityBalanceSummary) -> Self {
        Self {
            facility_remaining: balance.facility_remaining(),
            disbursed: Disbursed {
                total: balance.total_disbursed(),
                outstanding: balance.disbursed_outstanding(),
                outstanding_payable: balance.disbursed_outstanding_payable(),
            },
            interest: Interest {
                total: balance.interest_posted(),
                outstanding: balance.interest_outstanding(),
                outstanding_payable: balance.interest_outstanding_payable(),
            },
            outstanding: balance.total_outstanding(),
            outstanding_payable: balance.total_outstanding_payable(),
            due_outstanding: balance.total_overdue(),
            collateral: balance.collateral(),
            payments_unapplied: balance.payments_unapplied(),
        }
    }
}

#[derive(SimpleObject)]
pub struct Disbursed {
    pub total: UsdCents,
    pub outstanding: UsdCents,
    pub outstanding_payable: UsdCents,
}

#[derive(SimpleObject)]
pub struct Interest {
    pub total: UsdCents,
    pub outstanding: UsdCents,
    pub outstanding_payable: UsdCents,
}
