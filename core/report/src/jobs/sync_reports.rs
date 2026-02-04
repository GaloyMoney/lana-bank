use async_trait::async_trait;
use job::*;
use serde::{Deserialize, Serialize};

use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    CoreReportEvent,
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

impl<E> Clone for SyncReportsJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    fn clone(&self) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SyncReportsJobExecutionState {
    run_id: Option<String>,
}

pub struct SyncReportsJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    dagster: dagster::Dagster,
    report_runs: ReportRunRepo<E>,
    reports: ReportRepo,
}

impl<E> SyncReportsJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new(
        dagster: dagster::Dagster,
        report_runs: ReportRunRepo<E>,
        reports: ReportRepo,
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
    type Config = SyncReportsJobConfig<E>;

    fn job_type(&self) -> JobType {
        SYNC_REPORTS_JOB_TYPE
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        let _config: SyncReportsJobConfig<E> = job.config()?;
        Ok(Box::new(SyncReportsJobRunner::new(
            self.dagster.clone(),
            self.report_runs.clone(),
            self.reports.clone(),
        )))
    }

    fn retry_on_error_settings(&self) -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct SyncReportsJobRunner<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    dagster: dagster::Dagster,
    report_runs: ReportRunRepo<E>,
    reports: ReportRepo,
}

#[async_trait]
impl<E> JobRunner for SyncReportsJobRunner<E>
where
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    #[record_error_severity]
    #[tracing::instrument(name = "core_reports.job.sync_reports.run", skip(self, current_job))]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<SyncReportsJobExecutionState>()?
            .unwrap_or_default();

        let response = self
            .dagster
            .graphql()
            .file_reports_runs(1, state.run_id.clone())
            .await?;

        let runs = match response.data.runs_or_error {
            dagster::graphql_client::RunsOrError::Runs(runs) => runs,
            dagster::graphql_client::RunsOrError::Error { message } => {
                tracing::error!("Error fetching runs from Dagster: {}", message);
                return Err(message.into());
            }
        };

        for run_result in runs.results {
            self.sync_run(&run_result).await?;

            state.run_id = Some(run_result.run_id);
            current_job.update_execution_state(&state).await?;
        }

        Ok(JobCompletion::Complete)
    }
}

impl<E> SyncReportsJobRunner<E>
where
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    pub fn new(
        dagster: dagster::Dagster,
        report_runs: ReportRunRepo<E>,
        reports: ReportRepo,
    ) -> Self {
        Self {
            dagster,
            report_runs,
            reports,
        }
    }

    /// Syncs a single Dagster run to the local database.
    /// Creates or updates the report run record and syncs associated reports if finished.
    pub async fn sync_run(
        &self,
        run_result: &dagster::graphql_client::RunResult,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let state: ReportRunState = run_result.status.clone().into();
        let run_type: ReportRunType = run_result.into();

        let existing = self
            .report_runs
            .find_by_external_id(&run_result.run_id)
            .await;

        let run_id = match existing {
            Ok(mut report_run) => {
                if report_run
                    .update_state(state, run_type, run_result.start_time)
                    .did_execute()
                {
                    let mut db = self.report_runs.begin_op().await?;
                    self.report_runs
                        .update_in_op(&mut db, &mut report_run)
                        .await?;
                    db.commit().await?;
                }
                report_run.id
            }
            Err(e) if e.was_not_found() => {
                let new_run = NewReportRun::builder()
                    .external_id(run_result.run_id.clone())
                    .state(state)
                    .run_type(run_type)
                    .start_time(run_result.start_time)
                    .build()?;

                let mut db = self.report_runs.begin_op().await?;
                let report_run = self.report_runs.create_in_op(&mut db, new_run).await?;
                db.commit().await?;
                report_run.id
            }
            Err(e) => return Err(e.into()),
        };

        if run_result.status.is_finished() {
            self.sync_reports_if_missing(&run_result.run_id, run_id)
                .await?;
        }

        Ok(())
    }

    async fn sync_reports_if_missing(
        &self,
        external_id: &str,
        run_id: crate::ReportRunId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let reports = self
            .reports
            .list_for_run_id_by_created_at(
                run_id,
                Default::default(),
                es_entity::ListDirection::Descending,
            )
            .await?;

        if !reports.entities.is_empty() {
            return Ok(());
        }

        let dagster_reports = self.dagster.graphql().get_logs_for_run(external_id).await?;

        for dagster_report in dagster_reports {
            let files: Vec<crate::report::ReportFile> =
                dagster_report.files.into_iter().map(|f| f.into()).collect();

            let report_external_id = format!(
                "{}_{}_{}",
                external_id, dagster_report.norm, dagster_report.name
            );

            let new_report = NewReport::builder()
                .external_id(report_external_id)
                .run_id(run_id)
                .name(dagster_report.name)
                .norm(dagster_report.norm)
                .files(files)
                .build()?;

            let mut db = self.reports.begin_op().await?;
            self.reports.create_in_op(&mut db, new_report).await?;
            db.commit().await?;
        }

        Ok(())
    }
}

pub type SyncReportsJobSpawner<E> = JobSpawner<SyncReportsJobConfig<E>>;
