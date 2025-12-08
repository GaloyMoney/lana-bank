use async_trait::async_trait;
use chrono::Utc;
use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, RetrySettings,
};
use serde::{Deserialize, Serialize};

use outbox::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    event::CoreReportEvent,
    report::{NewReport, ReportRepo},
    report_run::{NewReportRun, ReportRunRepo, ReportRunState, ReportRunType},
};

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
    dagster: dagster::Dagster,
    report_runs: ReportRunRepo<E>,
    reports: ReportRepo<E>,
}

impl<E> SyncReportsJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new(
        dagster: dagster::Dagster,
        report_runs: ReportRunRepo<E>,
        reports: ReportRepo<E>,
    ) -> Self {
        Self {
            dagster,
            report_runs,
            reports,
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
            dagster: self.dagster.clone(),
            report_runs: self.report_runs.clone(),
            reports: self.reports.clone(),
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
    dagster: dagster::Dagster,
    report_runs: ReportRunRepo<E>,
    reports: ReportRepo<E>,
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
        let limit = 50;

        let response = self
            .dagster
            .graphql()
            .file_reports_runs(limit, None)
            .await?;

        let runs = match response.data.runs_or_error {
            dagster::graphql_client::RunsOrError::Runs(runs) => runs,
            dagster::graphql_client::RunsOrError::Error { message } => {
                tracing::error!("Error fetching runs from Dagster: {}", message);
                return Err(message.into());
            }
        };

        for run_result in runs.results.iter().rev() {
            let existing_run = self
                .report_runs
                .find_by_external_id(&run_result.run_id)
                .await;

            match existing_run {
                Ok(_) => {
                    continue;
                }
                Err(e) if e.was_not_found() => {
                    let state: ReportRunState = run_result.status.clone().into();

                    let new_run = NewReportRun::builder()
                        .external_id(run_result.run_id.clone())
                        .execution_date(Utc::now())
                        .state(state)
                        .run_type(ReportRunType::Scheduled)
                        .build()?;

                    let mut db = self.report_runs.begin_op().await?;
                    let report_run = self.report_runs.create_in_op(&mut db, new_run).await?;
                    db.commit().await?;

                    if run_result.status.is_finished() {
                        let dagster_reports = self
                            .dagster
                            .graphql()
                            .get_logs_for_run(&run_result.run_id)
                            .await?;

                        for dagster_report in dagster_reports {
                            let files: Vec<crate::report::ReportFile> =
                                dagster_report.files.into_iter().map(|f| f.into()).collect();

                            let external_id = format!(
                                "{}_{}_{}",
                                run_result.run_id, dagster_report.norm, dagster_report.name
                            );

                            let new_report = NewReport::builder()
                                .external_id(external_id)
                                .run_id(report_run.id)
                                .name(dagster_report.name)
                                .norm(dagster_report.norm)
                                .files(files)
                                .build()?;

                            let mut db = self.reports.begin_op().await?;
                            self.reports.create_in_op(&mut db, new_report).await?;
                            db.commit().await?;
                        }
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(JobCompletion::Complete)
    }
}
