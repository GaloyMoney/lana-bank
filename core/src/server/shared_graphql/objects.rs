use async_graphql::*;

use super::primitives::{Satoshis, UsdCents};

#[derive(SimpleObject)]
pub struct SuccessPayload {
    pub success: bool,
}

impl From<()> for SuccessPayload {
    fn from(_: ()) -> Self {
        SuccessPayload { success: true }
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
}

#[derive(SimpleObject)]
pub struct Interest {
    pub total: Total,
    pub outstanding: Outstanding,
}
