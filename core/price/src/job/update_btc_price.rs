use async_trait::async_trait;
use tracing::instrument;

use std::{marker::PhantomData, time::Duration};

use job::*;
use outbox::{EphemeralEventType, Outbox, OutboxEventMarker};

use crate::{bfx_client::*, event::PriceUpdated};

const PRICE_UPDATED_EVENT_NAME: &str = "core.price.updated-btc-price";
const UPDATE_BTC_PRICE_JOB_TYPE: JobType = JobType::new("core.price.update-btc-price");
const UPDATE_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateBtcPriceJobConfig<E>
where
    E: OutboxEventMarker<PriceUpdated>,
{
    #[serde(skip)]
    _phantom: PhantomData<E>,
}

impl<E> UpdateBtcPriceJobConfig<E>
where
    E: OutboxEventMarker<PriceUpdated>,
{
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<E> JobConfig for UpdateBtcPriceJobConfig<E>
where
    E: OutboxEventMarker<PriceUpdated>,
{
    type Initializer = UpdateBtcPriceInit<E>;
}

#[derive(Clone)]
pub struct UpdateBtcPriceInit<E>
where
    E: OutboxEventMarker<PriceUpdated>,
{
    outbox: Outbox<E>,
    bfx_client: BfxClient,
}

impl<E> UpdateBtcPriceInit<E>
where
    E: OutboxEventMarker<PriceUpdated>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
            bfx_client: BfxClient::new(),
        }
    }
}

impl<E> JobInitializer for UpdateBtcPriceInit<E>
where
    E: OutboxEventMarker<PriceUpdated>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        UPDATE_BTC_PRICE_JOB_TYPE
    }

    fn retry_on_error_settings() -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UpdateBtcPriceJobRunner {
            outbox: self.outbox.clone(),
            bfx_client: self.bfx_client.clone(),
            _phantom: PhantomData,
        }))
    }
}

pub struct UpdateBtcPriceJobRunner<E>
where
    E: OutboxEventMarker<PriceUpdated>,
{
    outbox: Outbox<E>,
    bfx_client: BfxClient,
    _phantom: PhantomData<E>,
}

impl<E> UpdateBtcPriceJobRunner<E>
where
    E: OutboxEventMarker<PriceUpdated>,
{
    #[instrument(
        name = "core.price.update_btc_price.fetch_and_publish",
        skip_all,
        fields(event_type = PRICE_UPDATED_EVENT_NAME)
    )]
    async fn fetch_and_publish_price(&self) -> Result<(), Box<dyn std::error::Error>> {
        let price = usd_cents_per_btc_cached(&self.bfx_client).await?;
        self.outbox
            .publish_ephemeral(
                EphemeralEventType::new(PRICE_UPDATED_EVENT_NAME),
                PriceUpdated { price },
            )
            .await?;
        Ok(())
    }
}

#[async_trait]
impl<E> JobRunner for UpdateBtcPriceJobRunner<E>
where
    E: OutboxEventMarker<PriceUpdated>,
{
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        self.fetch_and_publish_price().await?;
        if std::env::var("BFX_LOCAL_PRICE").is_ok() {
            Ok(JobCompletion::Complete)
        } else {
            Ok(JobCompletion::RescheduleIn(UPDATE_INTERVAL))
        }
    }
}
