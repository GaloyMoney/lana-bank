use crate::primitives::{Satoshis, UsdCents};

use async_graphql::*;

#[derive(SimpleObject)]
pub struct BtcBalance {
    pub btc_balance: Satoshis,
}

#[derive(SimpleObject)]
pub struct UsdBalance {
    pub usd_balance: UsdCents,
}
