#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod airflow;
mod report;
mod report_run;

pub mod error;
mod event;
mod jobs;
mod primitives;
mod publisher;

use publisher::*;

use audit::AuditSvc;
use authz::PermissionCheck;
use cloud_storage::Storage;
use job::Jobs;
use outbox::{Outbox, OutboxEventMarker};

use crate::airflow::reports_api_client::ReportsApiClient;

pub use crate::airflow::config::AirflowConfig;
pub use error::ReportError;
pub use event::CoreReportEvent;
pub use primitives::*;
pub use report::{File, Report, ReportsByCreatedAtCursor};
pub use report_run::{ReportRun, ReportRunState, ReportRunType, ReportRunsByCreatedAtCursor};

use jobs::{
    FindNewReportRunJobConfig, FindNewReportRunJobInit, MonitorReportRunJobInit,
    TriggerReportRunJobInit,
};
use report::Reports;
use report_run::ReportRuns;

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::event::CoreReportEvent;
}

pub struct CoreReports<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    authz: Perms,
    reports: Reports<Perms, E>,
    report_runs: ReportRuns<Perms, E>,
    airflow: ReportsApiClient,
    storage: Storage,
    jobs: Jobs,
}

impl<Perms, E> Clone for CoreReports<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            reports: self.reports.clone(),
            report_runs: self.report_runs.clone(),
            airflow: self.airflow.clone(),
            storage: self.storage.clone(),
            jobs: self.jobs.clone(),
        }
    }
}

impl<Perms, E> CoreReports<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreReportAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<ReportObject>,
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Perms,
        airflow_config: AirflowConfig,
        outbox: &Outbox<E>,
        jobs: &Jobs,
        storage: &Storage,
    ) -> Result<Self, ReportError> {
        let publisher = ReportPublisher::new(outbox);
        let airflow = ReportsApiClient::new(airflow_config.clone());
        let reports = Reports::init(pool, authz.clone(), &publisher, outbox).await?;
        let report_runs = ReportRuns::init(pool, authz.clone(), &publisher, outbox).await?;

        jobs.add_initializer(MonitorReportRunJobInit::new(
            airflow.clone(),
            report_runs.clone(),
            reports.clone(),
        ));
        jobs.add_initializer(TriggerReportRunJobInit::new(
            airflow.clone(),
            report_runs.clone(),
            jobs.clone(),
        ));
        jobs.add_initializer_and_spawn_unique(
            FindNewReportRunJobInit::new(airflow.clone(), report_runs.clone(), jobs.clone()),
            FindNewReportRunJobConfig::new(),
        )
        .await?;

        Ok(Self {
            authz: authz.clone(),
            storage: storage.clone(),
            airflow,
            reports,
            report_runs,
            jobs: jobs.clone(),
        })
    }

    pub async fn find_all_reports(
        &self,
        ids: &[ReportId],
    ) -> Result<std::collections::HashMap<ReportId, Report>, ReportError> {
        self.reports.find_all(ids).await.map_err(ReportError::from)
    }

    pub async fn find_all_report_runs(
        &self,
        ids: &[ReportRunId],
    ) -> Result<std::collections::HashMap<ReportRunId, ReportRun>, ReportError> {
        self.report_runs
            .find_all(ids)
            .await
            .map_err(ReportError::from)
    }

    pub async fn list_report_runs(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<ReportRunsByCreatedAtCursor>,
    ) -> Result<es_entity::PaginatedQueryRet<ReportRun, ReportRunsByCreatedAtCursor>, ReportError>
    {
        self.authz
            .enforce_permission(
                sub,
                ReportObject::all_report_runs(),
                CoreReportAction::REPORT_READ,
            )
            .await?;
        Ok(self
            .report_runs
            .list_by_created_at(query, es_entity::ListDirection::Descending)
            .await?)
    }

    pub async fn list_reports_for_run(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        run_id: ReportRunId,
    ) -> Result<Vec<Report>, ReportError> {
        self.authz
            .enforce_permission(
                sub,
                ReportObject::all_reports(),
                CoreReportAction::REPORT_READ,
            )
            .await?;
        self.reports
            .list_for_run_id(run_id)
            .await
            .map_err(ReportError::from)
    }
}
