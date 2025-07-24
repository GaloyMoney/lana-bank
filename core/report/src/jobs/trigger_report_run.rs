use async_trait::async_trait;
use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, Jobs,
    RetrySettings,
};
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use outbox::OutboxEventMarker;

use crate::{
    airflow::reports_api_client::ReportsApiClient,
    event::CoreReportEvent,
    primitives::{CoreReportAction, ReportObject},
    report_run::{NewReportRun, ReportRuns},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct TriggerReportRunJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> TriggerReportRunJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> JobConfig for TriggerReportRunJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreReportAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<ReportObject>,
    E: OutboxEventMarker<CoreReportEvent>,
{
    type Initializer = TriggerReportRunJobInit<Perms, E>;
}

pub struct TriggerReportRunJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub airflow: ReportsApiClient,
    pub report_runs: ReportRuns<Perms, E>,
    pub jobs: Jobs,
}

impl<Perms, E> TriggerReportRunJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new(airflow: ReportsApiClient, report_runs: ReportRuns<Perms, E>, jobs: Jobs) -> Self {
        Self {
            airflow,
            report_runs,
            jobs,
        }
    }
}

const TRIGGER_REPORT_RUN_JOB_TYPE: JobType = JobType::new("trigger-report-run");

impl<Perms, E> JobInitializer for TriggerReportRunJobInit<Perms, E>
where
    Perms: PermissionCheck + Send + Sync + 'static,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreReportAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<ReportObject>,
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    fn job_type() -> JobType {
        TRIGGER_REPORT_RUN_JOB_TYPE
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        let _config: TriggerReportRunJobConfig<Perms, E> = job.config()?;
        Ok(Box::new(TriggerReportRunJobRunner {
            airflow: self.airflow.clone(),
            report_runs: self.report_runs.clone(),
            jobs: self.jobs.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct TriggerReportRunJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    airflow: ReportsApiClient,
    report_runs: ReportRuns<Perms, E>,
    jobs: Jobs,
}

#[async_trait]
impl<Perms, E> JobRunner for TriggerReportRunJobRunner<Perms, E>
where
    Perms: PermissionCheck + Send + Sync + 'static,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreReportAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<ReportObject>,
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    #[tracing::instrument(
        name = "core_reports.find_new_report_run.run",
        skip(self, _current_job),
        err
    )]
    async fn run(
        &self,
        mut _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let run_id = self.airflow.generate_report().await?.run_id;

        let report_run = self
            .report_runs
            .repo()
            .create(
                NewReportRun::builder()
                    .external_id(run_id)
                    .build()
                    .expect("Failed to create NewReportRun"),
            )
            .await?;

        let mut db = self.report_runs.repo().begin_op().await?;
        self.jobs
            .create_and_spawn_in_op(
                &mut db,
                job::JobId::new(),
                super::monitor_report_run::MonitorReportRunJobConfig::<Perms, E>::new(
                    report_run.id,
                ),
            )
            .await?;
        db.commit().await?;

        Ok(JobCompletion::Complete)
    }
}
