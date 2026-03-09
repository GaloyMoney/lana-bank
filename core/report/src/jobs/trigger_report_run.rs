use async_trait::async_trait;
use chrono::NaiveDate;
use job::*;
use serde::{Deserialize, Serialize};

use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::dagster_adapter::DagsterReportAdapter;
use crate::report_run::ReportRunRepo;
use crate::{CoreReportEvent, find_report_definition};

use super::{SyncReportsJobConfig, SyncReportsJobSpawner};

const SYNC_REPORTS_DELAY_SECS: u64 = 10;

#[derive(Debug, Serialize, Deserialize)]
pub struct TriggerReportRunJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    report_definition_id: String,
    #[serde(default)]
    as_of_date: Option<NaiveDate>,
    _phantom: std::marker::PhantomData<E>,
}

impl<E> Clone for TriggerReportRunJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    fn clone(&self) -> Self {
        Self {
            report_definition_id: self.report_definition_id.clone(),
            as_of_date: self.as_of_date,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E> TriggerReportRunJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new(report_definition_id: String, as_of_date: Option<NaiveDate>) -> Self {
        Self {
            report_definition_id,
            as_of_date,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct TriggerReportRunJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    dagster_adapter: DagsterReportAdapter,
    sync_reports_spawner: SyncReportsJobSpawner<E>,
    report_runs: ReportRunRepo<E>,
}

impl<E> TriggerReportRunJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new(
        dagster_adapter: DagsterReportAdapter,
        sync_reports_spawner: SyncReportsJobSpawner<E>,
        report_runs: ReportRunRepo<E>,
    ) -> Self {
        Self {
            dagster_adapter,
            sync_reports_spawner,
            report_runs,
        }
    }
}

const TRIGGER_REPORT_RUN_JOB_TYPE: JobType = JobType::new("task.trigger-report-run");

impl<E> JobInitializer for TriggerReportRunJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    type Config = TriggerReportRunJobConfig<E>;

    fn job_type(&self) -> JobType {
        TRIGGER_REPORT_RUN_JOB_TYPE
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        let config: TriggerReportRunJobConfig<E> = job.config()?;
        Ok(Box::new(TriggerReportRunJobRunner {
            dagster_adapter: self.dagster_adapter.clone(),
            sync_reports_spawner: self.sync_reports_spawner.clone(),
            report_runs: self.report_runs.clone(),
            config,
        }))
    }

    fn retry_on_error_settings(&self) -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct TriggerReportRunJobRunner<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    dagster_adapter: DagsterReportAdapter,
    sync_reports_spawner: SyncReportsJobSpawner<E>,
    report_runs: ReportRunRepo<E>,
    config: TriggerReportRunJobConfig<E>,
}

#[async_trait]
impl<E> JobRunner for TriggerReportRunJobRunner<E>
where
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    #[record_error_severity]
    #[tracing::instrument(
        name = "core_reports.job.trigger_report_run.run",
        skip(self, _current_job)
    )]
    async fn run(
        &self,
        mut _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let report_definition = find_report_definition(&self.config.report_definition_id)
            .ok_or_else(|| {
                format!(
                    "unknown report definition '{}'",
                    self.config.report_definition_id
                )
            })?;

        let dagster_run_id = self
            .dagster_adapter
            .launch_report_run(report_definition, self.config.as_of_date)
            .await?;

        tracing::info!("Successfully triggered file report run: {}", dagster_run_id);

        let schedule_at =
            chrono::Utc::now() + chrono::Duration::seconds(SYNC_REPORTS_DELAY_SECS as i64);
        let mut db = self.report_runs.begin_op().await?;
        self.sync_reports_spawner
            .spawn_at_in_op(
                &mut db,
                JobId::new(),
                SyncReportsJobConfig::<E>::new(Some(dagster_run_id)),
                schedule_at,
            )
            .await?;
        db.commit().await?;

        Ok(JobCompletion::Complete)
    }
}

pub type TriggerReportRunJobSpawner<E> = JobSpawner<TriggerReportRunJobConfig<E>>;
