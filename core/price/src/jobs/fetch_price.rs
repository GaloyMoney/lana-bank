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

#[tracing::instrument(
    name = "core.price.fetch_price_from_single",
    skip_all,
    fields(provider_name)
)]
async fn fetch_price_from_single(
    provider: &crate::provider::PriceProvider,
) -> Result<PriceOfOneBTC, PriceProviderError> {
    tracing::Span::current().record("provider_name", provider.name.as_str());
    match provider.config() {
        PriceProviderConfig::Bitfinex => {
            let client = bfx_client::BfxClient::new();
            let tick = client.btc_usd_tick().await?;
            let usd_cents = money::UsdCents::try_from_usd(tick.last_price)?;
            Ok(PriceOfOneBTC::new(usd_cents))
        }
        PriceProviderConfig::ManualPrice { usd_cents_per_btc } => Ok(PriceOfOneBTC::new(
            money::UsdCents::from(*usd_cents_per_btc),
        )),
    }
}

#[record_error_severity]
#[tracing::instrument(name = "core.price.fetch_price", skip(providers))]
async fn fetch_price_from_providers(
    providers: &PriceProviderRepo,
) -> Result<PriceOfOneBTC, PriceProviderError> {
    let result = providers
        .list_for_active_by_id(true, Default::default(), Default::default())
        .await?;
    let active_providers = result.entities;
    if active_providers.is_empty() {
        return Err(PriceProviderError::NoActiveProviders);
    }

    let mut prices = Vec::new();
    for provider in &active_providers {
        match fetch_price_from_single(provider).await {
            Ok(price) => prices.push(price),
            Err(e) => {
                tracing::warn!(
                    provider_name = %provider.name,
                    error = %e,
                    "Failed to fetch price from provider, skipping"
                );
            }
        }
    }

    if prices.is_empty() {
        return Err(PriceProviderError::AllProvidersFailed);
    }

    prices.sort();
    let median = prices[prices.len() / 2];
    Ok(median)
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
            let price = fetch_price_from_providers(&self.providers).await?;
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
