use async_trait::async_trait;
use job::{
    CurrentJob, Job, JobCompletion, JobInitializer, JobRunner, JobSpawner, JobType, RetrySettings,
};
use obix::out::{Outbox, OutboxEventMarker};
use serde::{Deserialize, Serialize};
use tokio::select;

use std::time::Duration;

use domain_config::ExposedDomainConfigsReadOnly;

use crate::{
    ClosingSchedule,
    config::{ClosingTime, Timezone},
    event::CoreTimeEvent,
};

const SLEEP_INTERVAL: Duration = Duration::from_hours(1);
const DELTA: Duration = Duration::from_mins(5);

#[derive(Deserialize, Serialize)]
pub(crate) struct EndOfDayProducerJobConfig<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    pub _phantom: std::marker::PhantomData<E>,
}

pub(crate) struct EndOfDayProducerJobInit<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    outbox: Outbox<E>,
    domain_configs: ExposedDomainConfigsReadOnly,
}

impl<E> EndOfDayProducerJobInit<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    pub(crate) fn new(outbox: &Outbox<E>, domain_configs: &ExposedDomainConfigsReadOnly) -> Self {
        Self {
            outbox: outbox.clone(),
            domain_configs: domain_configs.clone(),
        }
    }
}

pub(crate) const END_OF_DAY_PRODUCER_JOB: JobType =
    JobType::new("cron.core-time-event.end-of-day-producer");

impl<E> JobInitializer for EndOfDayProducerJobInit<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    type Config = EndOfDayProducerJobConfig<E>;

    fn job_type(&self) -> JobType {
        END_OF_DAY_PRODUCER_JOB
    }

    fn init(
        &self,
        _: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(EndOfDayProducerJobRunner {
            outbox: self.outbox.clone(),
            domain_configs: self.domain_configs.clone(),
        }))
    }

    fn retry_on_error_settings(&self) -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub(crate) struct EndOfDayProducerJobRunner<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    outbox: Outbox<E>,
    domain_configs: ExposedDomainConfigsReadOnly,
}

#[async_trait]
impl<E> JobRunner for EndOfDayProducerJobRunner<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        loop {
            let closing_time_config = self
                .domain_configs
                .get_without_audit::<ClosingTime>()
                .await?;
            let timezone_config = self.domain_configs.get_without_audit::<Timezone>().await?;
            let closing_time = closing_time_config.value();
            let timezone = timezone_config.value();
            let clock = current_job.clock().clone();

            let schedule = ClosingSchedule::new(timezone, closing_time, &clock);

            let duration_until_close = match schedule.duration_until_close() {
                Ok(duration) => duration,
                Err(err) => {
                    // recalculate if past date returned
                    tracing::warn!(
                        job_id = %current_job.id(),
                        job_type = %END_OF_DAY_PRODUCER_JOB,
                        closing_time = %schedule.next_closing(),
                        error = %err,
                        "Closing time is in the past, recalculating"
                    );
                    continue;
                }
            };

            // TODO: Make consumers of this event idempotent
            let sleep_duration = if duration_until_close <= DELTA {
                let mut op = current_job.begin_op().await?;
                self.outbox
                    .publish_persisted_in_op(
                        &mut op,
                        CoreTimeEvent::EndOfDay {
                            day: schedule.next_closing_day(),
                            closing_time: schedule.next_closing(),
                            timezone: schedule.timezone(),
                        },
                    )
                    .await?;
                op.commit().await?;

                tracing::info!(
                    job_id = %current_job.id(),
                    job_type = %END_OF_DAY_PRODUCER_JOB,
                    day = %schedule.next_closing_day(),
                    closing_time = %schedule.next_closing(),
                    "End of day event published"
                );

                duration_until_close
            } else {
                std::cmp::min(SLEEP_INTERVAL, duration_until_close - DELTA)
            };

            tracing::info!(
                job_id = %current_job.id(),
                job_type = %END_OF_DAY_PRODUCER_JOB,
                ?sleep_duration,
                closing_time = %schedule.next_closing(),
                "Sleeping"
            );

            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %END_OF_DAY_PRODUCER_JOB,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }

                _ = clock.sleep(sleep_duration) => {}
            }
        }
    }
}
