use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use job::*;
use outbox::{EphemeralOutboxEvent, Outbox, OutboxEventMarker};

use crate::{CorePriceEvent, PRICE_UPDATED_EVENT_TYPE, PriceCache};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdatePriceInCacheJobConfig<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    pub _phantom: std::marker::PhantomData<E>,
}

impl<E> JobConfig for UpdatePriceInCacheJobConfig<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    type Initializer = UpdatePriceInCacheInit<E>;
}

pub struct UpdatePriceInCacheInit<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    outbox: Outbox<E>,
    cache: PriceCache,
}

impl<E> UpdatePriceInCacheInit<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(outbox: &Outbox<E>, cache: &PriceCache) -> Self {
        Self {
            outbox: outbox.clone(),
            cache: cache.clone(),
        }
    }
}

const UPDATE_PRICE_IN_CACHE_JOB_TYPE: JobType =
    JobType::new("outbox.core-price.update-price-in-cache");

impl<E> JobInitializer for UpdatePriceInCacheInit<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    fn job_type() -> JobType {
        UPDATE_PRICE_IN_CACHE_JOB_TYPE
    }

    fn init(&self, _job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UpdatePriceInCacheJobRunner::<E> {
            outbox: self.outbox.clone(),
            cache: self.cache.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct UpdatePriceInCacheJobRunner<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    outbox: Outbox<E>,
    cache: PriceCache,
}

impl<E> UpdatePriceInCacheJobRunner<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    async fn process_message(
        &self,
        message: &EphemeralOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if message.event_type.as_str() == PRICE_UPDATED_EVENT_TYPE {
            if let Some(CorePriceEvent::PriceUpdated { price }) = message.payload.as_event() {
                self.cache.set_price(*price);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl<E> JobRunner for UpdatePriceInCacheJobRunner<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut stream = self.outbox.listen_ephemeral().await?;

        while let Some(message) = stream.next().await {
            self.process_message(message.as_ref()).await?;
        }

        Ok(JobCompletion::RescheduleNow)
    }
}
