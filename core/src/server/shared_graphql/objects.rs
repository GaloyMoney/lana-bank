use crate::primitives::{SignedSatoshis, SignedUsdCents};

use async_graphql::*;

#[derive(SimpleObject)]
pub struct BtcBalance {
    pub btc_balance: SignedSatoshis,
}

#[derive(SimpleObject)]
pub struct UsdBalance {
    pub usd_balance: SignedUsdCents,
}

#[derive(SimpleObject)]
pub struct SuccessPayload {
    pub success: bool,
}

impl From<()> for SuccessPayload {
    fn from(_: ()) -> Self {
        SuccessPayload { success: true }
    }
}
