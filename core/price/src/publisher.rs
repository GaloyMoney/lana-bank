use chrono::{DateTime, Utc};
use outbox::{EphemeralEventType, Outbox, OutboxEventMarker};

use crate::{CorePriceEvent, PriceError, PriceOfOneBTC};

const BTC_USD_PRICE_UPDATED_EVENT_TYPE: &str = "btc_usd_price_updated";

pub struct PricePublisher<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for PricePublisher<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> PricePublisher<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    pub async fn publish_btc_usd_price_update(
        &self,
        price: PriceOfOneBTC,
        recorded_at: DateTime<Utc>,
    ) -> Result<(), PriceError> {
        let event = CorePriceEvent::BtcUsdPriceUpdated { price, recorded_at };
        self.outbox
            .publish_ephemeral(
                EphemeralEventType::new(BTC_USD_PRICE_UPDATED_EVENT_TYPE),
                event,
            )
            .await?;
        Ok(())
    }
}
