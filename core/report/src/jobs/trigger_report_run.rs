use async_trait::async_trait;
use chrono::NaiveDate;
use job::*;
use serde::{Deserialize, Serialize};

use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::report_run::ReportRunRepo;
use crate::{
    AS_OF_DATE_TAG_KEY, CoreReportEvent, MANUAL_SINGLE_REPORT_TAG_KEY,
    REPORT_DEFINITION_ID_TAG_KEY, REPORT_NAME_TAG_KEY, REPORT_NORM_TAG_KEY, find_report_definition,
};
use dagster::Dagster;

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
        let config: TriggerReportRunJobConfig<E> = job.config()?;
        Ok(Box::new(TriggerReportRunJobRunner {
            dagster: self.dagster.clone(),
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
    dagster: Dagster,
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

        let response = self
            .dagster
            .graphql()
            .launch_file_report_run(dagster::graphql_client::LaunchFileReportRunInput {
                asset_selection: report_definition
                    .asset_selection_paths()
                    .into_iter()
                    .map(dagster::graphql_client::AssetKeyInput::from_path)
                    .collect(),
                as_of_date: self.config.as_of_date,
                tags: build_run_tags(report_definition, self.config.as_of_date),
            })
            .await?;

        match response.data.launch_run {
            dagster::graphql_client::LaunchRunResult::LaunchRunSuccess { run } => {
                if let Some(details) = run {
                    tracing::info!("Successfully triggered file report run: {}", details.run_id);

                    let schedule_at = chrono::Utc::now()
                        + chrono::Duration::seconds(SYNC_REPORTS_DELAY_SECS as i64);
                    let mut db = self.report_runs.begin_op().await?;
                    self.sync_reports_spawner
                        .spawn_at_in_op(
                            &mut db,
                            JobId::new(),
                            SyncReportsJobConfig::<E>::new(Some(details.run_id.clone())),
                            schedule_at,
                        )
                        .await?;
                    db.commit().await?;

                    Ok(JobCompletion::Complete)
                } else {
                    Err("No run details returned from Dagster".into())
                }
            }
            dagster::graphql_client::LaunchRunResult::Error => {
                Err("Failed to launch pipeline in Dagster".into())
            }
        }
    }
}

pub type TriggerReportRunJobSpawner<E> = JobSpawner<TriggerReportRunJobConfig<E>>;

fn build_run_tags(
    report_definition: &crate::ReportDefinition,
    as_of_date: Option<NaiveDate>,
) -> Vec<dagster::graphql_client::ExecutionTag> {
    let mut tags = vec![
        dagster::graphql_client::ExecutionTag {
            key: MANUAL_SINGLE_REPORT_TAG_KEY.to_string(),
            value: "true".to_string(),
        },
        dagster::graphql_client::ExecutionTag {
            key: REPORT_DEFINITION_ID_TAG_KEY.to_string(),
            value: report_definition.report_definition_id(),
        },
        dagster::graphql_client::ExecutionTag {
            key: REPORT_NORM_TAG_KEY.to_string(),
            value: report_definition.norm.clone(),
        },
        dagster::graphql_client::ExecutionTag {
            key: REPORT_NAME_TAG_KEY.to_string(),
            value: report_definition.friendly_name.clone(),
        },
    ];

    if let Some(as_of_date) = as_of_date {
        tags.push(dagster::graphql_client::ExecutionTag {
            key: AS_OF_DATE_TAG_KEY.to_string(),
            value: as_of_date.to_string(),
        });
    }

    tags
}
