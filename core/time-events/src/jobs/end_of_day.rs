use async_trait::async_trait;
use chrono::NaiveDate;
use job::{
    CurrentJob, Job, JobCompletion, JobInitializer, JobRunner, JobSpawner, JobType, RetrySettings,
};
use obix::out::{Outbox, OutboxEventMarker};
use serde::{Deserialize, Serialize};

use domain_config::ExposedDomainConfigsReadOnly;

use crate::{
    ClosingSchedule,
    config::{ClosingTime, Timezone},
    event::CoreTimeEvent,
};

#[derive(Default, Clone, Serialize, Deserialize)]
struct EndOfDayProducerState {
    last_published_day: Option<NaiveDate>,
}

#[derive(Deserialize, Serialize)]
pub struct EndOfDayProducerJobConfig<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    pub _phantom: std::marker::PhantomData<E>,
}

pub struct EndOfDayProducerJobInit<E>
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
    pub fn new(outbox: &Outbox<E>, domain_configs: &ExposedDomainConfigsReadOnly) -> Self {
        Self {
            outbox: outbox.clone(),
            domain_configs: domain_configs.clone(),
        }
    }
}

pub const END_OF_DAY_PRODUCER_JOB: JobType =
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

pub struct EndOfDayProducerJobRunner<E>
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
        let mut state = current_job
            .execution_state::<EndOfDayProducerState>()?
            .unwrap_or_default();

        let closing_time_config = self
            .domain_configs
            .get_without_audit::<ClosingTime>()
            .await?;
        let timezone_config = self.domain_configs.get_without_audit::<Timezone>().await?;
        let closing_time = closing_time_config.value();
        let timezone = timezone_config.value();
        let clock = current_job.clock().clone();

        let schedule = ClosingSchedule::new(timezone, closing_time, &clock);
        let most_recent_closing_day = schedule.next_closing_day() - chrono::Days::new(1);

        match state.last_published_day {
            None => {
                state.last_published_day = Some(most_recent_closing_day);
                current_job.update_execution_state(&state).await?;

                tracing::info!(
                    job_id = %current_job.id(),
                    job_type = %END_OF_DAY_PRODUCER_JOB,
                    day = %most_recent_closing_day,
                    "Initialized last_published_day"
                );
            }
            Some(last_day) if last_day < most_recent_closing_day => {
                let mut day = last_day + chrono::Days::new(1);
                while day <= most_recent_closing_day {
                    let closing_dt = ClosingSchedule::closing_for_day(timezone, closing_time, day);

                    let mut op = current_job.begin_op().await?;
                    self.outbox
                        .publish_persisted_in_op(
                            &mut op,
                            CoreTimeEvent::EndOfDay {
                                day,
                                closing_time: closing_dt,
                                timezone,
                            },
                        )
                        .await?;
                    state.last_published_day = Some(day);
                    current_job
                        .update_execution_state_in_op(&mut op, &state)
                        .await?;
                    op.commit().await?;

                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %END_OF_DAY_PRODUCER_JOB,
                        day = %day,
                        closing_time = %closing_dt,
                        "End of day event published"
                    );

                    day = day + chrono::Days::new(1);
                }
            }
            _ => {
                tracing::debug!(
                    job_id = %current_job.id(),
                    job_type = %END_OF_DAY_PRODUCER_JOB,
                    "Already caught up"
                );
            }
        }

        Ok(JobCompletion::RescheduleAt(schedule.next_closing()))
    }
}
