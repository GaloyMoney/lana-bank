use async_trait::async_trait;
use job::{
    CurrentJob, Job, JobCompletion, JobInitializer, JobRunner, JobSpawner, JobType, RetrySettings,
};
use obix::out::{Outbox, OutboxEventMarker};
use serde::{Deserialize, Serialize};
use tokio::select;

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use domain_config::ExposedDomainConfigs;

use crate::{
    ClosingSchedule,
    config::{ClosingTimeConfig, TimezoneConfig},
    event::CoreTimeEvent,
};

#[derive(Deserialize, Serialize)]
pub struct EndOfDayBroadcastJobConfig<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    pub _phantom: std::marker::PhantomData<E>,
}

pub struct EndOfDayBroadcastJobInit<E, Perms>
where
    E: OutboxEventMarker<CoreTimeEvent>,
    Perms: PermissionCheck,
{
    outbox: Outbox<E>,
    domain_configs: ExposedDomainConfigs<Perms>,
}

impl<E, Perms> EndOfDayBroadcastJobInit<E, Perms>
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

pub const END_OF_DAY_BROADCAST_JOB: JobType =
    JobType::new("cron.core-time-event.end-of-day-broadcast");

impl<E, Perms> JobInitializer for EndOfDayBroadcastJobInit<E, Perms>
where
    E: OutboxEventMarker<CoreTimeEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<domain_config::DomainConfigAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<domain_config::DomainConfigObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject: SystemSubject,
{
    type Config = EndOfDayBroadcastJobConfig<E>;

    fn job_type(&self) -> JobType {
        END_OF_DAY_BROADCAST_JOB
    }

    fn init(
        &self,
        _: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(EndOfDayBroadcastJobRunner {
            outbox: self.outbox.clone(),
            domain_configs: self.domain_configs.clone(),
        }))
    }

    fn retry_on_error_settings(&self) -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct EndOfDayBroadcastJobRunner<E, Perms>
where
    E: OutboxEventMarker<CoreTimeEvent>,
    Perms: PermissionCheck,
{
    outbox: Outbox<E>,
    domain_configs: ExposedDomainConfigs<Perms>,
}

#[async_trait]
impl<E, Perms> JobRunner for EndOfDayBroadcastJobRunner<E, Perms>
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
            let closing_time_config = self
                .domain_configs
                .get::<ClosingTimeConfig>(&system_sub)
                .await?;
            let timezone_config = self
                .domain_configs
                .get::<TimezoneConfig>(&system_sub)
                .await?;
            let closing_time = closing_time_config.value().expect("has default").value;
            let timezone = timezone_config.value().expect("has default").value;

            let schedule = ClosingSchedule::new(timezone, closing_time);
            let current_time = current_job.clock().now();
            let closing_time = schedule.next_closing_from(current_time);

            let duration_until_close = match (closing_time - current_time).to_std() {
                Ok(duration) => duration,
                // continue if past date returned and recalculate, not likely
                Err(err) => {
                    tracing::warn!(
                        job_id = %current_job.id(),
                        job_type = %END_OF_DAY_BROADCAST_JOB,
                        current_time = %current_time,
                        closing_time = %closing_time,
                        error = %err,
                        "Closing time is in the past, recalculating"
                    );
                    continue;
                }
            };
            let clock = current_job.clock().clone();
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %END_OF_DAY_BROADCAST_JOB,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }

                // TODO: Bring in some special buffer logic as it evolves, like:
                // 1. Consumer uses the time_stamp from this event, not its own clock for processing EndOfDay tasks
                // 2. Current sleep can wake early and does a loop of find-grained sleeps to publish "on time" and can even publish event early
                // making the consumer prioritize this
                _ = clock.sleep(duration_until_close) => {
                    tracing::debug!(job_id = %current_job.id(), "Sleep completed, continuing");
                    let mut op = current_job.begin_op().await?;
                    self.outbox
                        .publish_persisted_in_op(&mut op, CoreTimeEvent::EndOfDay { closing_time })
                        .await?;
                    op.commit().await?;

                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %END_OF_DAY_BROADCAST_JOB,
                        closing_time = %closing_time,
                        "End of day event published"
                    );
                }
            }
        }
    }
}
