use async_trait::async_trait;
use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, RetrySettings,
};
use serde::{Deserialize, Serialize};

use outbox::OutboxEventMarker;

use crate::{event::CoreReportEvent, report_run::*};
use dagster::Dagster;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SyncReportsJobConfig<E>
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

#[derive(Default, Clone, Serialize, Deserialize)]
struct SyncReportsJobExecutionState {
    last_run_id: Option<String>,
}

pub struct SyncReportsJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub dagster: Dagster,
    pub report_run_repo: ReportRunRepo<E>,
}

impl<E> SyncReportsJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new(dagster: Dagster, report_run_repo: ReportRunRepo<E>) -> Self {
        Self {
            dagster,
            report_run_repo,
        }
    }
}

const SYNC_REPORTS_JOB_TYPE: JobType = JobType::new("cron.sync-reports");

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
            report_run_repo: self.report_run_repo.clone(),
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
    dagster: Dagster,
    report_run_repo: ReportRunRepo<E>,
}

#[async_trait]
impl<E> JobRunner for SyncReportsJobRunner<E>
where
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    #[tracing::instrument(
        name = "core_reports.job.sync_reports.run",
        skip(self, current_job),
        err
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<SyncReportsJobExecutionState>()?
            .unwrap_or_default();

        if let Ok(response) = self
            .dagster
            .client()
            .check_for_new_reports(state.last_run_id.clone())
            .await
        {
            println!("response: {:?}", response);
            // for run in response.runs.into_iter() {
            //     // Create the report run if it doesn't exist
            //     match self
            //         .report_run_repo
            //         .create(
            //             NewReportRun::builder()
            //                 .external_id(run.run_id.clone())
            //                 .build()
            //                 .expect("Failed to create NewReportRun"),
            //         )
            //         .await
            //     {
            //         Ok(mut report_run) => {
            //             // Update state for newly created run
            //             let new_state = ReportRunState::from(run.status);
            //             report_run.update_state_from_dagster(new_state, None, None);
            //             self.report_run_repo.update(&mut report_run).await?;
            //         }
            //         Err(e)
            //             if e.to_string()
            //                 .contains("duplicate key value violates unique constraint") =>
            //         {
            //             // Already exists, update if needed
            //             if let Ok(mut report_run) = self
            //                 .report_run_repo
            //                 .find_by_external_id(&run.run_id)
            //                 .await
            //             {
            //                 let new_state = ReportRunState::from(run.status);
            //                 if new_state != report_run.state {
            //                     report_run.update_state_from_dagster(new_state, None, None);
            //                     self.report_run_repo.update(&mut report_run).await?;
            //                 }
            //             }
            //         }
            //         Err(e) => {
            //             return Err(e.into());
            //         }
            //     };

            //     state.last_run_id = Some(run.run_id);
            //     current_job.update_execution_state(&state).await?;
            // }
        }

        Ok(JobCompletion::RescheduleIn(std::time::Duration::from_secs(
            60,
        )))
    }
}
