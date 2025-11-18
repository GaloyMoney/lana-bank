#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]
pub mod bfx_client;
pub mod error;
mod event;
pub mod jobs;
mod primitives;

use error::PriceError;
use futures::StreamExt;
use job::Jobs;
use outbox::{Outbox, OutboxEventMarker};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

pub use event::*;
pub use primitives::*;

#[derive(Clone)]
pub struct Price {
    inner: Arc<RwLock<PriceOfOneBTC>>,
    _handle: Arc<JoinHandle<()>>,
}

impl Price {
    pub async fn init<E>(jobs: &Jobs, outbox: Outbox<E>) -> Result<Self, PriceError>
    where
        E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
    {
        let price = Arc::new(RwLock::new(PriceOfOneBTC::ZERO));

        jobs.add_initializer_and_spawn_unique(
            jobs::get_price_from_bfx::GetPriceFromClientJobInit::<E>::new(&outbox),
            jobs::get_price_from_bfx::GetPriceFromClientJobConfig::<E> {
                _phantom: std::marker::PhantomData,
            },
        )
        .await
        .map_err(PriceError::JobError)?;

        let price_clone = Arc::clone(&price);

        let handle = tokio::spawn(async move {
            let mut stream = match outbox.listen_ephemeral().await {
                Ok(s) => s,
                Err(_) => return,
            };

            while let Some(message) = stream.next().await {
                if message.event_type.as_str() == PRICE_UPDATED_EVENT_TYPE {
                    if let Some(CorePriceEvent::PriceUpdated { price: new_price }) =
                        message.payload.as_event()
                    {
                        *price_clone.write().await = *new_price;
                    }
                }
            }
        });

        Ok(Self {
            inner: price,
            _handle: Arc::new(handle),
        })
    }

    pub async fn usd_cents_per_btc(&self) -> PriceOfOneBTC {
        *self.inner.read().await
    }
}
