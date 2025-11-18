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
    inner: Arc<RwLock<Option<PriceOfOneBTC>>>,
    _handle: Arc<JoinHandle<()>>,
}

impl Price {
    #[tracing::instrument(name = "core.price.init", skip(jobs, outbox), err)]
    pub async fn init<E>(jobs: &Jobs, outbox: Outbox<E>) -> Result<Self, PriceError>
    where
        E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
    {
        let price = Arc::new(RwLock::new(None));

        jobs.add_initializer_and_spawn_unique(
            jobs::get_price_from_bfx::GetPriceFromClientJobInit::<E>::new(&outbox),
            jobs::get_price_from_bfx::GetPriceFromClientJobConfig::<E> {
                _phantom: std::marker::PhantomData,
            },
        )
        .await
        .map_err(PriceError::JobError)?;

        let handle = Self::spawn_price_listener(outbox, Arc::clone(&price));

        Ok(Self {
            inner: price,
            _handle: Arc::new(handle),
        })
    }

    pub async fn usd_cents_per_btc(&self) -> Result<PriceOfOneBTC, PriceError> {
        self.inner
            .read()
            .await
            .copied()
            .ok_or(PriceError::PriceUnavailable)
    }

    fn spawn_price_listener<E>(
        outbox: Outbox<E>,
        price: Arc<RwLock<Option<PriceOfOneBTC>>>,
    ) -> JoinHandle<()>
    where
        E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
    {
        tokio::spawn(Self::listen_for_price_updates(outbox, price))
    }

    #[tracing::instrument(name = "core.price.listen_for_updates", skip(outbox, price))]
    async fn listen_for_price_updates<E>(outbox: Outbox<E>, price: Arc<RwLock<Option<PriceOfOneBTC>>>) 
    where
        E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
    {
        let mut stream = match outbox.listen_ephemeral().await {
            Ok(s) => s,
            Err(error) => {
                tracing::error!(?error, "failed to listen for price updates from outbox");
                return;
            }
        };

        while let Some(message) = stream.next().await {
            match message.payload.as_event() {
                Some(CorePriceEvent::PriceUpdated { price: new_price }) => {
                    *price.write().await = Some(*new_price);
                }
                None => {
                    tracing::warn!(
                        event_type = %message.event_type.as_str(),
                        "failed to deserialize CorePriceEvent from ephemeral outbox payload"
                    );
                }
            }
        }

        tracing::info!("price outbox listener stream ended");
    }
}
