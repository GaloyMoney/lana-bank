use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::{select, time::Duration};
use tracing_macros::record_error_severity;

use job::*;
use obix::out::{Outbox, OutboxEventMarker};

use crate::{
    CorePriceEvent, PRICE_UPDATED_EVENT_TYPE, PriceOfOneBTC,
    provider::{PriceProviderConfig, PriceProviderRepo, error::PriceProviderError},
};

const PRICE_UPDATE_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Clone, Serialize, Deserialize)]
pub struct FetchPriceJobConfig<E>
where
    E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
{
    #[serde(skip)]
    pub _phantom: std::marker::PhantomData<E>,
}

pub struct FetchPriceJobInit<E>
where
    E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
{
    providers: PriceProviderRepo,
    outbox: Outbox<E>,
}

impl<E> FetchPriceJobInit<E>
where
    E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
{
    pub(crate) fn new(providers: &PriceProviderRepo, outbox: &Outbox<E>) -> Self {
        Self {
            providers: providers.clone(),
            outbox: outbox.clone(),
        }
    }
}

const FETCH_PRICE_JOB_TYPE: JobType = JobType::new("cron.core-price.fetch-price");

impl<E> JobInitializer for FetchPriceJobInit<E>
where
    E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
{
    type Config = FetchPriceJobConfig<E>;
    fn job_type(&self) -> JobType {
        FETCH_PRICE_JOB_TYPE
    }

    fn init(
        &self,
        _job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(FetchPriceJobRunner::<E> {
            providers: self.providers.clone(),
            outbox: self.outbox.clone(),
        }))
    }

    fn retry_on_error_settings(&self) -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct FetchPriceJobRunner<E>
where
    E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
{
    providers: PriceProviderRepo,
    outbox: Outbox<E>,
}

#[record_error_severity]
#[tracing::instrument(name = "core.price.fetch_price", skip(providers))]
async fn fetch_price_from_provider(
    providers: &PriceProviderRepo,
) -> Result<PriceOfOneBTC, PriceProviderError> {
    let result = providers
        .list_by_id(Default::default(), Default::default())
        .await?;
    let provider = result
        .entities
        .into_iter()
        .next()
        .ok_or(PriceProviderError::Sqlx(sqlx::Error::RowNotFound))?;
    let config = provider.config();
    match config {
        PriceProviderConfig::Bitfinex => {
            let client = bfx_client::BfxClient::new();
            let tick = client.btc_usd_tick().await?;
            let usd_cents = money::UsdCents::try_from_usd(tick.last_price)?;
            Ok(PriceOfOneBTC::new(usd_cents))
        }
    }
}

#[async_trait]
impl<E> JobRunner for FetchPriceJobRunner<E>
where
    E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        loop {
            let price = fetch_price_from_provider(&self.providers).await?;
            self.outbox
                .publish_ephemeral(
                    PRICE_UPDATED_EVENT_TYPE,
                    CorePriceEvent::PriceUpdated {
                        price,
                        timestamp: current_job.clock().now(),
                    },
                )
                .await?;

            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %FETCH_PRICE_JOB_TYPE,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                _ = tokio::time::sleep(PRICE_UPDATE_INTERVAL) => {
                    tracing::debug!(job_id = %current_job.id(), "Sleep completed, continuing");
                }
            }
        }
    }
}
