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
pub(crate) struct FindNewReportRunJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    _phantom: std::marker::PhantomData<E>,
}

impl<E> FindNewReportRunJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E> JobConfig for FindNewReportRunJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    type Initializer = FindNewReportRunJobInit<E>;
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct FindNewReportRunJobExecutionState {
    run_id: Option<String>,
}

pub struct FindNewReportRunJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub airflow: Airflow,
    pub report_run_repo: ReportRunRepo<E>,
    pub jobs: Jobs,
}

impl<E> FindNewReportRunJobInit<E>
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

const FIND_NEW_REPORT_RUN_JOB_TYPE: JobType = JobType::new("find-new-report-run");

impl<E> JobInitializer for FindNewReportRunJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    fn job_type() -> JobType {
        FIND_NEW_REPORT_RUN_JOB_TYPE
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        let _config: FindNewReportRunJobConfig<E> = job.config()?;
        Ok(Box::new(FindNewReportRunJobRunner {
            airflow: self.airflow.clone(),
            report_run_repo: self.report_run_repo.clone(),
            jobs: self.jobs.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct FindNewReportRunJobRunner<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    airflow: Airflow,
    report_run_repo: ReportRunRepo<E>,
    jobs: Jobs,
}

#[async_trait]
impl<E> JobRunner for FindNewReportRunJobRunner<E>
where
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    #[tracing::instrument(
        name = "core_reports.find_new_report_run.run",
        skip(self, current_job),
        err
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<FindNewReportRunJobExecutionState>()?
            .unwrap_or_default();

        let next_runs = self
            .airflow
            .reports()
            .list_runs(Some(1), state.run_id)
            .await?;

        for run in next_runs.into_iter() {
            let report_run = match self
                .report_run_repo
                .create(
                    NewReportRun::builder()
                        .external_id(run.run_id.clone())
                        .build()
                        .expect("Failed to create NewReportRun"),
                )
                .await
            {
                Ok(report_run) => report_run,
                Err(e)
                    if e.to_string()
                        .contains("duplicate key value violates unique constraint") =>
                {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            };

            let mut db = self.report_run_repo.begin_op().await?;
            self.jobs
                .create_and_spawn_in_op(
                    &mut db,
                    job::JobId::new(),
                    super::monitor_report_run::MonitorReportRunJobConfig::<E>::new(report_run.id),
                )
                .await?;
            db.commit().await?;

            state.run_id = Some(run.run_id);
            current_job.update_execution_state(&state).await?;
        }

        Ok(JobCompletion::RescheduleIn(std::time::Duration::from_secs(
            60,
        )))
    }
}
