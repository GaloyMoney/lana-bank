use async_trait::async_trait;
use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType,
    RetrySettings,
};
use serde::{Deserialize, Serialize};

use outbox::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::event::CoreReportEvent;

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncReportsJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    _phantom: std::marker::PhantomData<E>,
}

impl<E> SyncReportsJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E> JobConfig for SyncReportsJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    type Initializer = SyncReportsJobInit<E>;
}

pub struct SyncReportsJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    _phantom: std::marker::PhantomData<E>,
}

impl<E> SyncReportsJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

const SYNC_REPORTS_JOB_TYPE: JobType = JobType::new("task.sync-reports");

impl<E> JobInitializer for SyncReportsJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    fn job_type() -> JobType {
        SYNC_REPORTS_JOB_TYPE
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        let _config: SyncReportsJobConfig<E> = job.config()?;
        Ok(Box::new(SyncReportsJobRunner {
            _phantom: std::marker::PhantomData::<E>,
        }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct SyncReportsJobRunner<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    _phantom: std::marker::PhantomData<E>,
}

#[async_trait]
impl<E> JobRunner for SyncReportsJobRunner<E>
where
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    #[record_error_severity]
    #[tracing::instrument(name = "core_reports.job.sync_reports.run", skip(self, _current_job))]
    async fn run(
        &self,
        mut _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        // Basic sync implementation - no business logic yet
        tracing::info!("Sync reports job triggered");

        Ok(JobCompletion::Complete)
    }
}
