pub mod error;

use error::PriceListenerError;
use futures::StreamExt;
use outbox::{Outbox, OutboxEventMarker};

pub use core_price::PriceOfOneBTC;
pub use core_price::PriceUpdated;

use serde::{Serialize, de::DeserializeOwned};

#[derive(Clone)]
pub struct PriceListener<E>
where
    E: Serialize + DeserializeOwned + Send + Sync + 'static + OutboxEventMarker<PriceUpdated>,
{
    outbox: Outbox<E>,
}

impl<E> PriceListener<E>
where
    E: Serialize + DeserializeOwned + Send + Sync + 'static + OutboxEventMarker<PriceUpdated>,
{
    pub fn init(outbox: Outbox<E>) -> Self {
        Self { outbox }
    }

    pub async fn usd_cents_per_btc(&self) -> Result<PriceOfOneBTC, PriceListenerError> {
        let mut stream = self.outbox.listen_ephemeral().await?;

        while let Some(message) = stream.next().await {
            if let Some(event) = message.as_event() {
                match &event.payload {
                    PriceUpdated { price, .. } => {
                        return Ok(*price);
                    }
                }
            }
        }

        Err(PriceListenerError::NoPriceAvailable)
    }
}
