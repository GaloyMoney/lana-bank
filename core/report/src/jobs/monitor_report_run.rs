use async_trait::async_trait;
use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, RetrySettings,
};
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use outbox::OutboxEventMarker;

use crate::{
    airflow::reports_api_client::ReportsApiClient,
    event::CoreReportEvent,
    primitives::*,
    report::{NewReport, Reports},
    report_run::{ReportRunState, ReportRuns},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitorReportRunJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    report_run_id: ReportRunId,
    _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> MonitorReportRunJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new(report_run_id: ReportRunId) -> Self {
        Self {
            report_run_id,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> JobConfig for MonitorReportRunJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreReportAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<ReportObject>,
    E: OutboxEventMarker<CoreReportEvent>,
{
    type Initializer = MonitorReportRunJobInit<Perms, E>;
}

pub struct MonitorReportRunJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub airflow: ReportsApiClient,
    pub report_runs: ReportRuns<Perms, E>,
    pub reports: Reports<Perms, E>,
}

impl<Perms, E> MonitorReportRunJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new(
        airflow: ReportsApiClient,
        report_runs: ReportRuns<Perms, E>,
        reports: Reports<Perms, E>,
    ) -> Self {
        Self {
            airflow,
            report_runs,
            reports,
        }
    }
}

const MONITOR_REPORT_RUN_JOB_TYPE: JobType = JobType::new("monitor-report-run");

impl<Perms, E> JobInitializer for MonitorReportRunJobInit<Perms, E>
where
    Perms: PermissionCheck + Send + Sync + 'static,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreReportAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<ReportObject>,
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    fn job_type() -> JobType {
        MONITOR_REPORT_RUN_JOB_TYPE
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        let config: MonitorReportRunJobConfig<Perms, E> = job.config()?;
        Ok(Box::new(MonitorReportRunJobRunner {
            config,
            airflow: self.airflow.clone(),
            report_runs: self.report_runs.clone(),
            reports: self.reports.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct MonitorReportRunJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    config: MonitorReportRunJobConfig<Perms, E>,
    airflow: ReportsApiClient,
    report_runs: ReportRuns<Perms, E>,
    reports: Reports<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for MonitorReportRunJobRunner<Perms, E>
where
    Perms: PermissionCheck + Send + Sync + 'static,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreReportAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<ReportObject>,
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    #[tracing::instrument(
        name = "core_reports.get_report_run.run",
        skip(self, _current_job),
        err
    )]
    async fn run(
        &self,
        mut _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut report_run = self
            .report_runs
            .repo()
            .find_by_id(self.config.report_run_id)
            .await?;

        let Some(details) = self.airflow.get_run(&report_run.external_id).await? else {
            return Ok(JobCompletion::RescheduleNow);
        };

        if report_run.state.map(Into::into) == Some(details.state) {
            return Ok(JobCompletion::RescheduleNow);
        }

        report_run.update_state(
            details.state.into(),
            details.run_type.into(),
            details.execution_date,
            details.start_date,
            details.end_date,
        );
        self.report_runs.repo().update(&mut report_run).await?;

        if matches!(
            report_run.state,
            Some(ReportRunState::Failed | ReportRunState::Success)
        ) {
            for report in details.reports {
                let new_report = NewReport::builder()
                    .external_id(report.id)
                    .run_id(report_run.id)
                    .name(report.name)
                    .norm(report.norm)
                    .files(report.files.into_iter().map(Into::into).collect())
                    .build()?;

                self.reports.repo().create(new_report).await?;
            }
            Ok(JobCompletion::Complete)
        } else {
            Ok(JobCompletion::RescheduleNow)
        }
    }
}
