#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod aggregator;
mod bfx_client;
pub mod error;
pub mod event;
mod primitives;
pub mod publisher;
pub mod sources;

use futures::StreamExt;
use outbox::{EphemeralEventType, Outbox, OutboxEventMarker};

use crate::error::PriceError;
use crate::event::CorePriceEvent;

pub use aggregator::PriceAggregator;
pub use event::CorePriceEvent as PriceEvent;
pub use primitives::PriceOfOneBTC;
pub use publisher::PricePublisher;
pub use sources::PriceClient;

const BTC_USD_PRICE_UPDATED_EVENT_TYPE: &str = "btc_usd_price_updated";

#[derive(Clone)]
pub struct Price<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Price<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    pub async fn usd_cents_per_btc(&self) -> Result<PriceOfOneBTC, PriceError> {
        let mut stream = self.outbox.listen_ephemeral().await?;
        let event_type = EphemeralEventType::new(BTC_USD_PRICE_UPDATED_EVENT_TYPE);

        while let Some(event) = stream.next().await {
            if event.event_type.as_str() == event_type.as_str() {
                if let Some(CorePriceEvent::BtcUsdPriceUpdated { price, .. }) =
                    event.payload.as_event()
                {
                    return Ok(*price);
                }
            }
        }

        Err(PriceError::MissingPrice)
    }
}
