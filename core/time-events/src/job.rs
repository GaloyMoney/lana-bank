use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use job::{CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType};
use outbox::{Outbox, OutboxEventMarker};

use crate::{TimeEvent, broadcaster::DailyClosingBroadcaster, config::TimeEventsConfig};

#[derive(Serialize, Deserialize)]
pub struct DailyClosingBroadcasterJobConfig<E>
where
    E: OutboxEventMarker<TimeEvent>,
{
    _phantom: std::marker::PhantomData<E>,
}

impl<E> Default for DailyClosingBroadcasterJobConfig<E>
where
    E: OutboxEventMarker<TimeEvent>,
{
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E> DailyClosingBroadcasterJobConfig<E>
where
    E: OutboxEventMarker<TimeEvent>,
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<E> JobConfig for DailyClosingBroadcasterJobConfig<E>
where
    E: OutboxEventMarker<TimeEvent> + Send + Sync + 'static,
{
    type Initializer = DailyClosingBroadcasterInit<E>;
}

pub struct DailyClosingBroadcasterInit<E>
where
    E: OutboxEventMarker<TimeEvent>,
{
    outbox: Outbox<E>,
    config: TimeEventsConfig,
}

impl<E> DailyClosingBroadcasterInit<E>
where
    E: OutboxEventMarker<TimeEvent>,
{
    pub fn new(outbox: &Outbox<E>, config: TimeEventsConfig) -> Self {
        Self {
            outbox: outbox.clone(),
            config,
        }
    }
}

const DAILY_CLOSING_BROADCASTER: JobType = JobType::new("time-events.daily-closing-broadcaster");

impl<E> JobInitializer for DailyClosingBroadcasterInit<E>
where
    E: OutboxEventMarker<TimeEvent> + Send + Sync + 'static,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        DAILY_CLOSING_BROADCASTER
    }

    #[instrument(name = "time_events.daily_closing_broadcaster_job.init", skip(self))]
    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        let broadcaster = DailyClosingBroadcaster::try_new(&self.outbox, self.config.clone())?;
        Ok(Box::new(DailyClosingBroadcasterJobRunner { broadcaster }))
    }
}

pub struct DailyClosingBroadcasterJobRunner<E>
where
    E: OutboxEventMarker<TimeEvent>,
{
    broadcaster: DailyClosingBroadcaster<E>,
}

#[async_trait]
impl<E> JobRunner for DailyClosingBroadcasterJobRunner<E>
where
    E: OutboxEventMarker<TimeEvent> + Send + Sync + 'static,
{
    #[instrument(name = "time_events.daily_closing_broadcaster_job.run", skip_all, err)]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        self.broadcaster.run().await;
        // This should never return, but if it does, reschedule immediately
        Ok(JobCompletion::RescheduleNow)
    }
}
