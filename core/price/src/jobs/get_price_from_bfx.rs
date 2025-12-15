use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::{select, time::Duration};

use job::*;
use outbox::{Outbox, OutboxEventMarker};
use std::sync::Arc;

use crate::{
    CorePriceEvent, PRICE_UPDATED_EVENT_TYPE, PriceOfOneBTC,
    bfx_client::{self, BfxClient},
};

const PRICE_UPDATE_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetPriceFromClientJobConfig<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    pub _phantom: std::marker::PhantomData<E>,
}

impl<E> JobConfig for GetPriceFromClientJobConfig<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    type Initializer = GetPriceFromClientJobInit<E>;
}

pub struct GetPriceFromClientJobInit<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    bfx_client: Arc<BfxClient>,
    outbox: Outbox<E>,
}

impl<E> GetPriceFromClientJobInit<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            bfx_client: Arc::new(BfxClient::new()),
            outbox: outbox.clone(),
        }
    }
}

const GET_PRICE_FROM_CLIENT_JOB_TYPE: JobType = JobType::new("cron.core-price.get-price-from-bfx");

impl<E> JobInitializer for GetPriceFromClientJobInit<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    fn job_type() -> JobType {
        GET_PRICE_FROM_CLIENT_JOB_TYPE
    }

    fn init(&self, _job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(GetPriceFromBfxJobRunner::<E> {
            bfx_client: self.bfx_client.clone(),
            outbox: self.outbox.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct GetPriceFromBfxJobRunner<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    bfx_client: Arc<BfxClient>,
    outbox: Outbox<E>,
}

#[async_trait]
impl<E> JobRunner for GetPriceFromBfxJobRunner<E>
where
    E: OutboxEventMarker<CorePriceEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        loop {
            let price: PriceOfOneBTC = bfx_client::fetch_price(self.bfx_client.as_ref()).await?;

            self.outbox
                .publish_ephemeral(
                    PRICE_UPDATED_EVENT_TYPE,
                    CorePriceEvent::PriceUpdated {
                        price,
                        timestamp: crate::time::now(),
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
