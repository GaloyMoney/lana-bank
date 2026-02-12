use async_graphql::*;

use crate::primitives::*;

#[derive(SimpleObject)]
pub(super) struct CreditFacilityBalance {
    facility_remaining: FacilityRemaining,
    disbursed: Disbursed,
    interest: Interest,
    outstanding: Outstanding,
    outstanding_payable: Outstanding,
    due_outstanding: Outstanding,
    collateral: CollateralBalance,
    payments_unapplied: PaymentsUnapplied,
}

impl From<lana_app::credit::CreditFacilityBalanceSummary> for CreditFacilityBalance {
    fn from(balance: lana_app::credit::CreditFacilityBalanceSummary) -> Self {
        Self {
            facility_remaining: FacilityRemaining {
                usd_balance: balance.facility_remaining(),
            },
            disbursed: Disbursed {
                total: Total {
                    usd_balance: balance.total_disbursed(),
                },
                outstanding: Outstanding {
                    usd_balance: balance.disbursed_outstanding(),
                },
                outstanding_payable: Outstanding {
                    usd_balance: balance.disbursed_outstanding_payable(),
                },
            },
            interest: Interest {
                total: Total {
                    usd_balance: balance.interest_posted(),
                },
                outstanding: Outstanding {
                    usd_balance: balance.interest_outstanding(),
                },
                outstanding_payable: Outstanding {
                    usd_balance: balance.interest_outstanding_payable(),
                },
            },
            outstanding: Outstanding {
                usd_balance: balance.total_outstanding(),
            },
            outstanding_payable: Outstanding {
                usd_balance: balance.total_outstanding_payable(),
            },
            due_outstanding: Outstanding {
                usd_balance: balance.total_overdue(),
            },
            collateral: CollateralBalance {
                btc_balance: balance.collateral(),
            },
            payments_unapplied: PaymentsUnapplied {
                usd_balance: balance.payments_unapplied(),
            },
        }
    }
}

#[derive(SimpleObject)]
pub(crate) struct CollateralBalance {
    pub btc_balance: Satoshis,
}

#[derive(SimpleObject)]
pub(crate) struct Outstanding {
    pub usd_balance: UsdCents,
}

#[derive(SimpleObject)]
pub(crate) struct Total {
    pub usd_balance: UsdCents,
}

#[derive(SimpleObject)]
pub(crate) struct FacilityRemaining {
    pub usd_balance: UsdCents,
}

#[derive(SimpleObject)]
pub(crate) struct Disbursed {
    pub total: Total,
    pub outstanding: Outstanding,
    pub outstanding_payable: Outstanding,
}

#[derive(SimpleObject)]
pub(crate) struct Interest {
    pub total: Total,
    pub outstanding: Outstanding,
    pub outstanding_payable: Outstanding,
}

#[derive(SimpleObject)]
pub(crate) struct PaymentsUnapplied {
    pub usd_balance: UsdCents,
}
