use async_graphql::{Context, Object, Subscription};
use futures::{StreamExt, stream::Stream};
use obix::out::OutboxEventMarker;

use super::*;
use lana_app::{app::LanaApp, price::CorePriceEvent};

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

#[derive(Default)]
pub struct PriceSubscription;

#[Subscription]
impl PriceSubscription {
    async fn realtime_price_updated(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<impl Stream<Item = RealtimePrice>> {
        let app = ctx.data_unchecked::<LanaApp>();

        let stream = app.outbox().listen_ephemeral();
        let updates = stream.filter_map(move |event| async move {
            let event: &CorePriceEvent = event.payload.as_event()?;
            match event {
                CorePriceEvent::PriceUpdated { price, .. } => Some(RealtimePrice::from(*price)),
            }
        });

        Ok(updates)
    }
}
