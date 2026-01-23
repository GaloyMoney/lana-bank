use async_trait::async_trait;
use job::{
    CurrentJob, Job, JobCompletion, JobInitializer, JobRunner, JobSpawner, JobType, RetrySettings,
};
use obix::out::{Outbox, OutboxEventMarker};
use serde::{Deserialize, Serialize};
use tokio::select;

use std::time::Duration;

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use domain_config::ExposedDomainConfigs;

use crate::{
    ClosingSchedule,
    config::{ClosingTime, Timezone},
    event::CoreTimeEvent,
};

const SLEEP_INTERVAL: Duration = Duration::from_hours(1);
const DELTA: Duration = Duration::from_mins(5);

#[derive(Deserialize, Serialize)]
pub struct EndOfDayProducerJobConfig<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    pub _phantom: std::marker::PhantomData<E>,
}

pub struct EndOfDayProducerJobInit<E, Perms>
where
    E: OutboxEventMarker<CoreTimeEvent>,
    Perms: PermissionCheck,
{
    outbox: Outbox<E>,
    domain_configs: ExposedDomainConfigs<Perms>,
}

impl<E, Perms> EndOfDayProducerJobInit<E, Perms>
where
    E: OutboxEventMarker<CoreTimeEvent>,
    Perms: PermissionCheck,
{
    pub fn new(outbox: &Outbox<E>, domain_configs: &ExposedDomainConfigs<Perms>) -> Self {
        Self {
            outbox: outbox.clone(),
            domain_configs: domain_configs.clone(),
        }
    }
}

pub const END_OF_DAY_PRODUCER_JOB: JobType =
    JobType::new("cron.core-time-event.end-of-day-producer");

impl<E, Perms> JobInitializer for EndOfDayProducerJobInit<E, Perms>
where
    E: OutboxEventMarker<CoreTimeEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<domain_config::DomainConfigAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<domain_config::DomainConfigObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject: SystemSubject,
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

pub struct EndOfDayProducerJobRunner<E, Perms>
where
    E: OutboxEventMarker<CoreTimeEvent>,
    Perms: PermissionCheck,
{
    outbox: Outbox<E>,
    domain_configs: ExposedDomainConfigs<Perms>,
}

#[async_trait]
impl<E, Perms> JobRunner for EndOfDayProducerJobRunner<E, Perms>
where
    E: OutboxEventMarker<CoreTimeEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<domain_config::DomainConfigAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<domain_config::DomainConfigObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject: SystemSubject,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let system_sub = <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system();

        loop {
            let closing_time_config = self.domain_configs.get::<ClosingTime>(&system_sub).await?;
            let timezone_config = self.domain_configs.get::<Timezone>(&system_sub).await?;
            let closing_time = closing_time_config
                .value()
                .expect("closing time must have a default")
                .parse::<chrono::NaiveTime>()?;
            let timezone = timezone_config
                .value()
                .expect("timezone must have a default")
                .parse::<chrono_tz::Tz>()?;
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
