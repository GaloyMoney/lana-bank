use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::{Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, RetrySettings};

use outbox::{Outbox, OutboxEventMarker};

use crate::{RealNow, TimeEvents, event::CoreTimeEvent};

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
    // time_events should also be generic over T for T: Now trait
    time_events: TimeEvents<RealNow>,
}

impl<E> EndOfDayJobInit<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    pub fn new(outbox: &Outbox<E>, time_events: &TimeEvents<RealNow>) -> Self {
        Self {
            outbox: outbox.clone(),
            time_events: time_events.clone(),
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
            // no turbofish?
            outbox: self.outbox.clone(),
            time_events: self.time_events.clone(),
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
    // wrap in arc for optimization?
    time_events: TimeEvents<RealNow>,
}

#[async_trait]
impl<E> JobRunner for EndOfDayJobRunner<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    async fn run(
        &self,
        current_job: job::CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let next_closing = self.time_events.next_closing_in_utc().await?;

        Ok(JobCompletion::RescheduleAt(next_closing))
    }
}
