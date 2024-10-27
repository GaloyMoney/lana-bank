use async_graphql::*;

use crate::primitives::UsdCents;

#[derive(SimpleObject)]
pub struct RealtimePrice {
    usd_cents_per_btc: UsdCents,
}

impl From<crate::primitives::PriceOfOneBTC> for RealtimePrice {
    fn from(price: crate::primitives::PriceOfOneBTC) -> Self {
        Self {
            usd_cents_per_btc: price.into_inner(),
        }
    }
}
