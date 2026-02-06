use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::{select, time::Duration};
use tracing_macros::record_error_severity;

use job::*;
use obix::out::{Outbox, OutboxEventMarker};
use std::sync::Arc;

use crate::{CorePriceEvent, PriceOfOneBTC, PRICE_UPDATED_EVENT_TYPE};

const PRICE_UPDATE_INTERVAL: Duration = Duration::from_secs(60);

#[record_error_severity]
#[tracing::instrument(name = "core.price.bfx_client.fetch_price", skip(client))]
pub async fn fetch_price(
    client: std::sync::Arc<bfx_client::BfxClient>,
) -> Result<PriceOfOneBTC, bfx_client::BfxClientError> {
    let tick = client.btc_usd_tick().await?;
    let usd_cents =
        money::UsdCents::try_from_usd(tick.last_price).map_err(bfx_client::BfxClientError::ConversionError)?;
    Ok(PriceOfOneBTC::new(usd_cents))
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GetPriceFromClientJobConfig<E>
where
    E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
{
    pub _phantom: std::marker::PhantomData<E>,
}

pub struct GetPriceFromClientJobInit<E>
where
    E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
{
    bfx_client: Arc<bfx_client::BfxClient>,
    outbox: Outbox<E>,
}

impl<E> GetPriceFromClientJobInit<E>
where
    E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            bfx_client: Arc::new(bfx_client::BfxClient::new()),
            outbox: outbox.clone(),
        }
    }
}

const GET_PRICE_FROM_CLIENT_JOB_TYPE: JobType = JobType::new("cron.core-price.get-price-from-bfx");

impl<E> JobInitializer for GetPriceFromClientJobInit<E>
where
    E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
{
    type Config = GetPriceFromClientJobConfig<E>;
    fn job_type(&self) -> JobType {
        GET_PRICE_FROM_CLIENT_JOB_TYPE
    }

    fn init(
        &self,
        _job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(GetPriceFromBfxJobRunner::<E> {
            bfx_client: self.bfx_client.clone(),
            outbox: self.outbox.clone(),
        }))
    }

    fn retry_on_error_settings(&self) -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct GetPriceFromBfxJobRunner<E>
where
    E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
{
    bfx_client: Arc<bfx_client::BfxClient>,
    outbox: Outbox<E>,
}

#[async_trait]
impl<E> JobRunner for GetPriceFromBfxJobRunner<E>
where
    E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        loop {
            let price: PriceOfOneBTC = fetch_price(self.bfx_client.clone()).await?;
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
                        job_type = %GET_PRICE_FROM_CLIENT_JOB_TYPE,
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
