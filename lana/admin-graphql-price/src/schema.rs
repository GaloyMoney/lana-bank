use async_graphql::{Context, Object};

use super::*;

#[derive(Default)]
pub struct PriceQuery;

#[Object]
impl PriceQuery {
    async fn realtime_price(&self, ctx: &Context<'_>) -> async_graphql::Result<RealtimePrice> {
        let app = ctx.data_unchecked::<lana_app::app::LanaApp>();
        let usd_cents_per_btc = app.price().usd_cents_per_btc().await;
        Ok(usd_cents_per_btc.into())
    }
}
