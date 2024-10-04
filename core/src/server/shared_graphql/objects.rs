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
