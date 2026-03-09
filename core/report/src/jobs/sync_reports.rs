use async_trait::async_trait;
use chrono::NaiveDate;
use job::*;
use serde::{Deserialize, Serialize};

use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    AS_OF_DATE_TAG_KEY, CoreReportEvent, REPORT_DEFINITION_ID_TAG_KEY, REPORT_NAME_TAG_KEY,
    REPORT_NORM_TAG_KEY,
    report::{NewReport, ReportRepo},
    report_run::{NewReportRun, ReportRunRepo, ReportRunState, ReportRunType, RequestedReport},
};

const SYNC_REPORTS_RETRY_SECS: u64 = 10;

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncReportsJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    #[serde(default)]
    target_run_id: Option<String>,
    _phantom: std::marker::PhantomData<E>,
}

impl<E> Clone for SyncReportsJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    fn clone(&self) -> Self {
        Self {
            target_run_id: self.target_run_id.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E> SyncReportsJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new(target_run_id: Option<String>) -> Self {
        Self {
            target_run_id,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct SyncReportsJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    dagster_adapter: crate::dagster_adapter::DagsterReportAdapter,
    report_runs: ReportRunRepo<E>,
    reports: ReportRepo,
}

impl<E> SyncReportsJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new(
        dagster_adapter: crate::dagster_adapter::DagsterReportAdapter,
        report_runs: ReportRunRepo<E>,
        reports: ReportRepo,
    ) -> Self {
        Self {
            dagster_adapter,
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
        let config: SyncReportsJobConfig<E> = job.config()?;
        Ok(Box::new(SyncReportsJobRunner::new_with_config(
            self.dagster_adapter.clone(),
            self.report_runs.clone(),
            self.reports.clone(),
            config,
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
    dagster_adapter: crate::dagster_adapter::DagsterReportAdapter,
    report_runs: ReportRunRepo<E>,
    reports: ReportRepo,
    config: SyncReportsJobConfig<E>,
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
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        if let Some(target_run_id) = self.config.target_run_id.as_deref() {
            let maybe_run = self
                .dagster_adapter
                .fetch_run(target_run_id)
                .await?;
            let Some(run_result) = maybe_run else {
                tracing::warn!(
                    target_run_id,
                    "Dagster run not visible yet, rescheduling sync"
                );
                return Ok(JobCompletion::RescheduleIn(std::time::Duration::from_secs(
                    SYNC_REPORTS_RETRY_SECS,
                )));
            };

            self.sync_run(&run_result).await?;

            if run_result.status.is_finished() {
                return Ok(JobCompletion::Complete);
            }

            tracing::debug!(
                target_run_id = %run_result.run_id,
                status = ?run_result.status,
                "Dagster run still in progress, rescheduling sync"
            );
            return Ok(JobCompletion::RescheduleIn(std::time::Duration::from_secs(
                SYNC_REPORTS_RETRY_SECS,
            )));
        }

        let response = self.dagster_adapter.fetch_recent_runs(1).await?;

        let runs = match response.data.runs_or_error {
            dagster::graphql_client::RunsOrError::Runs(runs) => runs,
            dagster::graphql_client::RunsOrError::Error { message } => {
                tracing::error!("Error fetching runs from Dagster: {}", message);
                return Err(message.into());
            }
        };

        for run_result in runs.results {
            self.sync_run(&run_result).await?;
        }

        Ok(JobCompletion::Complete)
    }
}

impl<E> SyncReportsJobRunner<E>
where
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    pub fn new(
        dagster_adapter: crate::dagster_adapter::DagsterReportAdapter,
        report_runs: ReportRunRepo<E>,
        reports: ReportRepo,
    ) -> Self {
        Self::new_with_config(
            dagster_adapter,
            report_runs,
            reports,
            SyncReportsJobConfig::<E>::new(None),
        )
    }

    pub fn new_with_config(
        dagster_adapter: crate::dagster_adapter::DagsterReportAdapter,
        report_runs: ReportRunRepo<E>,
        reports: ReportRepo,
        config: SyncReportsJobConfig<E>,
    ) -> Self {
        Self {
            dagster_adapter,
            report_runs,
            reports,
            config,
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
        let requested_report = requested_report_from_run(run_result);
        let requested_as_of_date = requested_as_of_date_from_run(run_result)?;

        let mut db = self.report_runs.begin_op().await?;
        let existing = self
            .report_runs
            .find_by_external_id_in_op(&mut db, &run_result.run_id)
            .await;

        let run_id = match existing {
            Ok(mut report_run) => {
                if report_run
                    .update_state(
                        state,
                        run_type,
                        run_result.start_time,
                        requested_report.clone(),
                        requested_as_of_date,
                    )
                    .did_execute()
                {
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
                    .requested_report(requested_report)
                    .requested_as_of_date(requested_as_of_date)
                    .build()?;

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
        let dagster_reports = self.dagster_adapter.fetch_reports_for_run(external_id).await?;

        let mut aggregated: std::collections::HashMap<
            (String, String),
            Vec<crate::report::ReportFile>,
        > = std::collections::HashMap::new();

        for dagster_report in dagster_reports {
            let key = (dagster_report.norm.clone(), dagster_report.name.clone());
            let files: Vec<crate::report::ReportFile> =
                dagster_report.files.into_iter().map(|f| f.into()).collect();
            aggregated.entry(key).or_default().extend(files);
        }

        for ((norm, name), files) in aggregated {
            let report_external_id = format!("{}_{}_{}", external_id, norm, name);

            if self
                .reports
                .find_by_external_id(&report_external_id)
                .await
                .is_ok()
            {
                continue;
            }

            let new_report = NewReport::builder()
                .external_id(report_external_id)
                .run_id(run_id)
                .name(name)
                .norm(norm)
                .files(files)
                .build()?;

            let mut db = self.reports.begin_op().await?;
            self.reports.create_in_op(&mut db, new_report).await?;
            db.commit().await?;
        }

        Ok(())
    }
}

fn requested_report_from_run(
    run_result: &dagster::graphql_client::RunResult,
) -> Option<RequestedReport> {
    let report_definition_id = tag_value(run_result, REPORT_DEFINITION_ID_TAG_KEY)?;
    let report_definition = crate::find_report_definition(report_definition_id);

    let norm = tag_value(run_result, REPORT_NORM_TAG_KEY)
        .map(str::to_owned)
        .or_else(|| report_definition.map(|definition| definition.norm.clone()))
        .unwrap_or_else(|| {
            report_definition_id
                .split_once('/')
                .map(|(norm, _)| norm.to_string())
                .unwrap_or_default()
        });

    let name = tag_value(run_result, REPORT_NAME_TAG_KEY)
        .map(str::to_owned)
        .or_else(|| report_definition.map(|definition| definition.friendly_name.clone()))
        .unwrap_or_else(|| {
            report_definition_id
                .rsplit('/')
                .next()
                .unwrap_or(report_definition_id)
                .to_string()
        });

    Some(RequestedReport {
        report_definition_id: report_definition_id.to_string(),
        norm,
        name,
    })
}

fn requested_as_of_date_from_run(
    run_result: &dagster::graphql_client::RunResult,
) -> Result<Option<NaiveDate>, chrono::ParseError> {
    match tag_value(run_result, AS_OF_DATE_TAG_KEY) {
        Some(as_of_date) => Ok(Some(NaiveDate::parse_from_str(as_of_date, "%Y-%m-%d")?)),
        None => Ok(None),
    }
}

fn tag_value<'a>(run_result: &'a dagster::graphql_client::RunResult, key: &str) -> Option<&'a str> {
    run_result
        .tags
        .iter()
        .find(|tag| tag.key == key)
        .map(|tag| tag.value.as_str())
}

pub type SyncReportsJobSpawner<E> = JobSpawner<SyncReportsJobConfig<E>>;
