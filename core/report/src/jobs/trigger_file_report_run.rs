use async_trait::async_trait;
use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, Jobs,
    RetrySettings,
};
use serde::{Deserialize, Serialize};

use outbox::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{event::CoreReportEvent, report_run::*};
use dagster::Dagster;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TriggerFileReportRunJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    _phantom: std::marker::PhantomData<E>,
}

impl<E> TriggerFileReportRunJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E> JobConfig for TriggerFileReportRunJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    type Initializer = TriggerFileReportRunJobInit<E>;
}

pub struct TriggerFileReportRunJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub dagster: Dagster,
    pub report_run_repo: ReportRunRepo<E>,
    pub jobs: Jobs,
}

impl<E> TriggerFileReportRunJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new(dagster: Dagster, report_run_repo: ReportRunRepo<E>, jobs: Jobs) -> Self {
        Self {
            dagster,
            report_run_repo,
            jobs,
        }
    }
}

const TRIGGER_FILE_REPORT_RUN_JOB_TYPE: JobType = JobType::new("task.trigger-file-report-run");

impl<E> JobInitializer for TriggerFileReportRunJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    fn job_type() -> JobType {
        TRIGGER_FILE_REPORT_RUN_JOB_TYPE
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        let _config: TriggerFileReportRunJobConfig<E> = job.config()?;
        Ok(Box::new(TriggerFileReportRunJobRunner {
            dagster: self.dagster.clone(),
            report_run_repo: self.report_run_repo.clone(),
            jobs: self.jobs.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct TriggerFileReportRunJobRunner<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    dagster: Dagster,
    report_run_repo: ReportRunRepo<E>,
    jobs: Jobs,
}

#[async_trait]
impl<E> JobRunner for TriggerFileReportRunJobRunner<E>
where
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    #[record_error_severity]
    #[tracing::instrument(
        name = "core_reports.job.trigger_file_report_run.run",
        skip(self, _current_job)
    )]
    async fn run(
        &self,
        mut _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let response = self.dagster.graphql().trigger_file_report_run().await?;

        match response.data.launch_pipeline_execution {
            dagster::graphql_client::LaunchPipelineResult::LaunchRunSuccess { run } => {
                if let Some(details) = run {
                    tracing::info!("Successfully triggered file report run: {}", details.run_id);
                    Ok(JobCompletion::Complete)
                } else {
                    Err("No run details returned from Dagster".into())
                }
            }
            dagster::graphql_client::LaunchPipelineResult::Error => {
                Err("Failed to launch pipeline in Dagster".into())
            }
        }
    }
}
