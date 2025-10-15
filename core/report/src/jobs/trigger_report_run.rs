use async_trait::async_trait;
use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, Jobs,
    RetrySettings,
};
use serde::{Deserialize, Serialize};

use outbox::OutboxEventMarker;

use crate::{event::CoreReportEvent, report_run::*};
use airflow::Airflow;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TriggerReportRunJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    _phantom: std::marker::PhantomData<E>,
}

impl<E> TriggerReportRunJobConfig<E>
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

impl<E> JobConfig for TriggerReportRunJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    type Initializer = TriggerReportRunJobInit<E>;
}

pub struct TriggerReportRunJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub airflow: Airflow,
    pub report_run_repo: ReportRunRepo<E>,
    pub jobs: Jobs,
}

impl<E> TriggerReportRunJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new(airflow: Airflow, report_run_repo: ReportRunRepo<E>, jobs: Jobs) -> Self {
        Self {
            airflow,
            report_run_repo,
            jobs,
        }
    }
}

const TRIGGER_REPORT_RUN_JOB_TYPE: JobType = JobType::new("task.trigger-report-run");

impl<E> JobInitializer for TriggerReportRunJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    fn job_type() -> JobType {
        TRIGGER_REPORT_RUN_JOB_TYPE
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        let _config: TriggerReportRunJobConfig<E> = job.config()?;
        Ok(Box::new(TriggerReportRunJobRunner {
            airflow: self.airflow.clone(),
            report_run_repo: self.report_run_repo.clone(),
            jobs: self.jobs.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct TriggerReportRunJobRunner<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    airflow: Airflow,
    report_run_repo: ReportRunRepo<E>,
    jobs: Jobs,
}

#[async_trait]
impl<E> JobRunner for TriggerReportRunJobRunner<E>
where
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    #[tracing::instrument(
        name = "core_reports.job.trigger_report_run.run",
        skip(self, _current_job),
        err
    )]
    async fn run(
        &self,
        mut _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let run_id = self.airflow.reports().generate_report().await?.run_id;

        let report_run = self
            .report_run_repo
            .create(
                NewReportRun::builder()
                    .external_id(run_id)
                    .build()
                    .expect("Failed to create NewReportRun"),
            )
            .await?;

        let mut db = self.report_run_repo.begin_op().await?;
        self.jobs
            .create_and_spawn_in_op(
                &mut db,
                job::JobId::new(),
                super::monitor_report_run::MonitorReportRunJobConfig::<E>::new(report_run.id),
            )
            .await?;
        db.commit().await?;

        Ok(JobCompletion::Complete)
    }
}
