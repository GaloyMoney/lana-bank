use async_trait::async_trait;
use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, RetrySettings,
};
use obix::out::{Outbox, OutboxEventMarker};
use serde::{Deserialize, Serialize};
use tokio::select;

use domain_config::DomainConfigs;

use crate::{
    ClosingSchedule,
    config::{ClosingTimeConfig, TimezoneConfig},
    event::CoreTimeEvent,
};

#[derive(Deserialize, Serialize)]
pub struct EndOfDayJobConfig<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    pub _phantom: std::marker::PhantomData<E>,
}

impl<E> JobConfig for EndOfDayJobConfig<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    type Initializer = EndOfDayJobInit<E>;
}

pub struct EndOfDayJobInit<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    outbox: Outbox<E>,
    domain_configs: DomainConfigs,
}

impl<E> EndOfDayJobInit<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    pub fn new(outbox: &Outbox<E>, domain_config: &DomainConfigs) -> Self {
        Self {
            outbox: outbox.clone(),
            domain_configs: domain_config.clone(),
        }
    }
}

pub const END_OF_DAY_JOB: JobType = JobType::new("cron.core-time-event.end-of-day");

impl<E> JobInitializer for EndOfDayJobInit<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    fn job_type() -> JobType {
        END_OF_DAY_JOB
    }

    fn init(&self, _job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(EndOfDayJobRunner {
            outbox: self.outbox.clone(),
            domain_configs: self.domain_configs.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct EndOfDayJobRunner<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    outbox: Outbox<E>,
    domain_configs: DomainConfigs,
}

#[async_trait]
impl<E> JobRunner for EndOfDayJobRunner<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        // include these queries in the loop?
        let tz_config = self
            .domain_configs
            .get_or_default::<TimezoneConfig>()
            .await?;

        let closing_time_config = self
            .domain_configs
            .get_or_default::<ClosingTimeConfig>()
            .await?;

        let schedule = ClosingSchedule::new(tz_config.timezone, closing_time_config.closing_time);

        loop {
            let current_time = crate::time::now();
            let closing_time = schedule.next_closing_from(current_time);

            let duration_until_close = match (closing_time - current_time).to_std() {
                Ok(duration) => duration,
                // sleep or continue if past date returned/make it impossible to return past dates?
                Err(err) => {
                    tracing::warn!(
                        job_id = %current_job.id(),
                        job_type = %END_OF_DAY_JOB,
                        current_time = %current_time,
                        closing_time = %closing_time,
                        error = %err,
                        "Closing time is in the past, recalculating"
                    );
                    continue;
                }
            };

            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %END_OF_DAY_JOB,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }

                _ = tokio::time::sleep(duration_until_close) => {
                    tracing::debug!(job_id = %current_job.id(), "Sleep completed, continuing");
                    let mut op = self.outbox.begin_op().await?;
                    self.outbox
                        .publish_persisted_in_op(&mut op, CoreTimeEvent::EndOfDay { closing_time })
                        .await?;
                    op.commit().await?;

                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %END_OF_DAY_JOB,
                        closing_time = %closing_time,
                        "End of day event published"
                    );
                }
            }
        }
    }
}
