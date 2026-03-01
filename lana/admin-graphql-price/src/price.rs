use async_graphql::*;

use lana_app::primitives::UsdCents;

#[derive(SimpleObject)]
pub struct RealtimePrice {
    usd_cents_per_btc: UsdCents,
}

impl From<lana_app::primitives::PriceOfOneBTC> for RealtimePrice {
    fn from(price: lana_app::primitives::PriceOfOneBTC) -> Self {
        Self {
            usd_cents_per_btc: price.into_inner(),
        }
    }
}
