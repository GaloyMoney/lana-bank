use async_trait::async_trait;
use job::*;
use serde::{Deserialize, Serialize};

use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::event::CoreReportEvent;
use crate::report_run::ReportRunRepo;
use dagster::Dagster;

use super::{SyncReportsJobConfig, SyncReportsJobSpawner};

const SYNC_REPORTS_DELAY_SECS: u64 = 10;

#[derive(Debug, Serialize, Deserialize)]
pub struct TriggerReportRunJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    _phantom: std::marker::PhantomData<E>,
}

impl<E> Clone for TriggerReportRunJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    fn clone(&self) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
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

pub struct TriggerReportRunJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    dagster: Dagster,
    sync_reports_spawner: SyncReportsJobSpawner<E>,
    report_runs: ReportRunRepo<E>,
}

impl<E> TriggerReportRunJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new(
        dagster: Dagster,
        sync_reports_spawner: SyncReportsJobSpawner<E>,
        report_runs: ReportRunRepo<E>,
    ) -> Self {
        Self {
            dagster,
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
        let _config: TriggerReportRunJobConfig<E> = job.config()?;
        Ok(Box::new(TriggerReportRunJobRunner {
            dagster: self.dagster.clone(),
            sync_reports_spawner: self.sync_reports_spawner.clone(),
            report_runs: self.report_runs.clone(),
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
    dagster: Dagster,
    sync_reports_spawner: SyncReportsJobSpawner<E>,
    report_runs: ReportRunRepo<E>,
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
        let response = self.dagster.graphql().trigger_file_report_run().await?;

        match response.data.launch_pipeline_execution {
            dagster::graphql_client::LaunchPipelineResult::LaunchRunSuccess { run } => {
                if let Some(details) = run {
                    tracing::info!("Successfully triggered file report run: {}", details.run_id);

                    let schedule_at = chrono::Utc::now()
                        + chrono::Duration::seconds(SYNC_REPORTS_DELAY_SECS as i64);
                    let mut db = self.report_runs.begin_op().await?;
                    self.sync_reports_spawner
                        .spawn_at_in_op(
                            &mut db,
                            JobId::new(),
                            SyncReportsJobConfig::<E>::new(),
                            schedule_at,
                        )
                        .await?;
                    db.commit().await?;

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

pub type TriggerReportRunJobSpawner<E> = JobSpawner<TriggerReportRunJobConfig<E>>;
