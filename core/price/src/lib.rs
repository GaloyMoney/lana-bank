#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]
pub mod bfx_client;
pub mod error;
mod event;
pub mod jobs;
mod primitives;
pub mod time;

use futures::StreamExt;
use job::Jobs;
use obix::out::{EphemeralOutboxEvent, Outbox, OutboxEventMarker};
use std::sync::Arc;
use tokio::{sync::watch, task::JoinHandle};
use tracing::Span;

use error::PriceError;

pub use event::*;
pub use primitives::*;

#[derive(Clone)]
pub struct Price {
    receiver: watch::Receiver<Option<PriceOfOneBTC>>,
    _handle: Arc<JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>>,
}

impl Price {
    #[tracing::instrument(name = "core.price.init", skip(jobs, outbox), err)]
    pub async fn init<E>(jobs: &mut Jobs, outbox: &Outbox<E>) -> Result<Self, PriceError>
    where
        E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
    {
        let spawner = jobs
            .add_initializer(jobs::get_price_from_bfx::GetPriceFromClientJobInit::<E>::new(outbox));
        spawner
            .spawn_unique(
                job::JobId::new(),
                jobs::get_price_from_bfx::GetPriceFromClientJobConfig::new(),
            )
            .await
            .map_err(PriceError::JobError)?;

        let (tx, rx) = watch::channel(None);

        let handle = Self::spawn_price_listener(tx, outbox.clone());

        Ok(Self {
            receiver: rx,
            _handle: Arc::new(handle),
        })
    }

    pub async fn usd_cents_per_btc(&self) -> PriceOfOneBTC {
        let mut rec = self.receiver.clone();
        loop {
            if let Some(res) = *rec.borrow() {
                return res;
            }
            let _ = rec.changed().await;
        }
    }

    fn spawn_price_listener<E>(
        tx: watch::Sender<Option<PriceOfOneBTC>>,
        outbox: Outbox<E>,
    ) -> JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>
    where
        E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
    {
        tokio::spawn(Self::listen_for_price_updates(tx, outbox))
    }

    #[tracing::instrument(name = "core.price.listen_for_updates", skip_all, err)]
    async fn listen_for_price_updates<E>(
        tx: watch::Sender<Option<PriceOfOneBTC>>,
        outbox: Outbox<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
    {
        let mut stream = outbox.listen_ephemeral();

        while let Some(message) = stream.next().await {
            Self::process_message(&tx, message.as_ref()).await?;
        }

        tracing::info!("price outbox listener stream ended");
        Ok(())
    }

    #[tracing::instrument(
        name = "core.price.listen_for_updates.process_message",
        parent = None,
        skip(tx, message),
        fields(event_type = tracing::field::Empty, handled = false, price = tracing::field::Empty, timestamp = tracing::field::Empty),
        err
    )]
    async fn process_message<E>(
        tx: &watch::Sender<Option<PriceOfOneBTC>>,
        message: &EphemeralOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
    {
        if let Some(CorePriceEvent::PriceUpdated {
            price: new_price,
            timestamp,
        }) = message.payload.as_event()
        {
            Span::current().record("handled", true);
            Span::current().record("event_type", "PriceUpdated");
            Span::current().record("price", tracing::field::display(new_price));
            Span::current().record("timestamp", tracing::field::debug(timestamp));
            tx.send(Some(*new_price))?;
        }

        Ok(())
    }
}
