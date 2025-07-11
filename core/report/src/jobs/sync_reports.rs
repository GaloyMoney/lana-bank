use async_trait::async_trait;
use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, RetrySettings,
};
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use outbox::OutboxEventMarker;

use crate::{
    airflow::ReportsApiClient,
    entity::NewReport,
    event::CoreReportEvent,
    primitives::{CoreReportAction, ReportId, ReportObject},
    repo::ReportRepo,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncReportsJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> SyncReportsJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> JobConfig for SyncReportsJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreReportAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<ReportObject>,
    E: OutboxEventMarker<CoreReportEvent>,
{
    type Initializer = SyncReportsJobInit<Perms, E>;
}

pub struct SyncReportsJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub reports_api_client: ReportsApiClient,
    pub repo: ReportRepo<E>,
    pub authz: Perms,
}

impl<Perms, E> SyncReportsJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new(reports_api_client: ReportsApiClient, repo: ReportRepo<E>, authz: Perms) -> Self {
        Self {
            reports_api_client,
            repo,
            authz,
        }
    }
}

const SYNC_REPORTS_JOB_TYPE: JobType = JobType::new("sync-reports");

impl<Perms, E> JobInitializer for SyncReportsJobInit<Perms, E>
where
    Perms: PermissionCheck + Send + Sync + 'static,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreReportAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<ReportObject>,
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    fn job_type() -> JobType {
        SYNC_REPORTS_JOB_TYPE
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        let _config: SyncReportsJobConfig<Perms, E> = job.config()?;
        Ok(Box::new(SyncReportsJobRunner {
            reports_api_client: self.reports_api_client.clone(),
            repo: self.repo.clone(),
            authz: self.authz.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct SyncReportsJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    reports_api_client: ReportsApiClient,
    repo: ReportRepo<E>,
    authz: Perms,
}

#[async_trait]
impl<Perms, E> JobRunner for SyncReportsJobRunner<Perms, E>
where
    Perms: PermissionCheck + Send + Sync + 'static,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreReportAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<ReportObject>,
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    #[tracing::instrument(name = "sync_reports_job.run", skip(self, _current_job), err)]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let dates = self.reports_api_client.get_report_dates().await?;
        for date in &dates {
            let reports = self.reports_api_client.get_reports_by_date(*date).await?;
            for path_in_bucket in reports {
                let report_id = ReportId::new();
                let audit_info = self
                    .authz
                    .audit()
                    .record_system_entry(
                        ReportObject::report(report_id),
                        CoreReportAction::REPORT_SYNC,
                    )
                    .await?;

                let new_report = NewReport::builder()
                    .id(report_id)
                    .date(date.and_hms_opt(0, 0, 0).unwrap().and_utc())
                    .path_in_bucket(path_in_bucket.clone())
                    .audit_info(audit_info)
                    .build()?;

                self.repo.create(new_report).await?;

                println!(
                    "Synced report for date {}: {}",
                    date.format("%Y-%m-%d"),
                    path_in_bucket
                );
            }
        }

        Ok(JobCompletion::RescheduleNow)
    }
}
