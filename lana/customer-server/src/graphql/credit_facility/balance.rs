use async_graphql::*;

use crate::primitives::*;

#[derive(SimpleObject)]
pub(super) struct CreditFacilityBalance {
    facility_remaining: FacilityRemaining,
    disbursed: Disbursed,
    interest: Interest,
    outstanding: Outstanding,
    due_outstanding: Outstanding,
    collateral: Collateral,
}

impl From<lana_app::credit::CreditFacilityBalance> for CreditFacilityBalance {
    fn from(balance: lana_app::credit::CreditFacilityBalance) -> Self {
        Self {
            facility_remaining: FacilityRemaining {
                usd_balance: balance.facility_remaining,
            },
            disbursed: Disbursed {
                total: Total {
                    usd_balance: balance.total_disbursed,
                },
                outstanding: Outstanding {
                    usd_balance: balance.disbursed_receivable,
                },
                due_outstanding: Outstanding {
                    usd_balance: balance.due_disbursed_receivable,
                },
            },
            interest: Interest {
                total: Total {
                    usd_balance: balance.total_interest_accrued,
                },
                outstanding: Outstanding {
                    usd_balance: balance.interest_receivable,
                },
                due_outstanding: Outstanding {
                    usd_balance: balance.due_interest_receivable,
                },
            },
            outstanding: Outstanding {
                usd_balance: balance.disbursed_receivable + balance.interest_receivable,
            },
            due_outstanding: Outstanding {
                usd_balance: balance.due_disbursed_receivable + balance.due_interest_receivable,
            },
            collateral: Collateral {
                btc_balance: balance.collateral,
            },
        }
    }
}

#[derive(SimpleObject)]
pub struct Collateral {
    pub btc_balance: Satoshis,
}

#[derive(SimpleObject)]
pub struct Outstanding {
    pub usd_balance: UsdCents,
}

#[derive(SimpleObject)]
pub struct Total {
    pub usd_balance: UsdCents,
}

#[derive(SimpleObject)]
pub struct FacilityRemaining {
    pub usd_balance: UsdCents,
}

#[derive(SimpleObject)]
pub struct Disbursed {
    pub total: Total,
    pub outstanding: Outstanding,
    pub due_outstanding: Outstanding,
}

#[derive(SimpleObject)]
pub struct Interest {
    pub total: Total,
    pub outstanding: Outstanding,
    pub due_outstanding: Outstanding,
}
