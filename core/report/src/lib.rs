#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod report;
pub mod report_run;

pub mod config;
pub mod error;
pub mod event;

mod jobs;
mod primitives;
mod publisher;

use audit::AuditSvc;
use authz::PermissionCheck;
use job::Jobs;
use obix::out::{Outbox, OutboxEventMarker};
use tracing_macros::*;

pub use config::*;
pub use error::ReportError;
pub use event::*;
pub use primitives::*;

use cloud_storage::Storage;
use publisher::ReportPublisher;

use jobs::{
    SyncReportsJobConfig, SyncReportsJobInit, SyncReportsJobSpawner, TriggerReportRunJobConfig,
    TriggerReportRunJobInit, TriggerReportRunJobSpawner,
};

use dagster::*;
pub use report::*;
pub use report_run::*;

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::event::CoreReportEvent;
    pub use crate::report::ReportEvent;
    pub use crate::report_run::ReportRunEvent;
}

pub struct CoreReports<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    authz: Perms,
    reports: ReportRepo<E>,
    report_runs: ReportRunRepo<E>,
    dagster: Dagster,
    storage: Storage,
    config: ReportConfig,
    sync_reports_spawner: SyncReportsJobSpawner<E>,
    trigger_report_run_spawner: TriggerReportRunJobSpawner<E>,
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
            dagster: self.dagster.clone(),
            storage: self.storage.clone(),
            config: self.config.clone(),
            sync_reports_spawner: self.sync_reports_spawner.clone(),
            trigger_report_run_spawner: self.trigger_report_run_spawner.clone(),
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
    #[record_error_severity]
    #[tracing::instrument(name = "report.init", skip_all)]
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Perms,
        config: ReportConfig,
        outbox: &Outbox<E>,
        storage: &Storage,
        jobs: &mut Jobs,
    ) -> Result<Self, ReportError> {
        let publisher = ReportPublisher::new(outbox);
        let dagster = Dagster::new(config.dagster.clone());
        let report_repo = ReportRepo::new(pool, &publisher);
        let report_run_repo = ReportRunRepo::new(pool, &publisher);

        let sync_reports_spawner = jobs.add_initializer(SyncReportsJobInit::<E>::new(
            dagster.clone(),
            report_run_repo.clone(),
            report_repo.clone(),
        ));
        let trigger_report_run_spawner = jobs.add_initializer(TriggerReportRunJobInit::<E>::new(
            dagster.clone(),
            sync_reports_spawner.clone(),
            report_run_repo.clone(),
        ));

        Ok(Self {
            authz: authz.clone(),
            storage: storage.clone(),
            dagster,
            reports: report_repo,
            report_runs: report_run_repo,
            config: config.clone(),
            sync_reports_spawner,
            trigger_report_run_spawner,
        })
    }

    #[record_error_severity]
    #[tracing::instrument(name = "report.find_all_reports", skip(self), fields(count = ids.len()))]
    pub async fn find_all_reports(
        &self,
        ids: &[ReportId],
    ) -> Result<std::collections::HashMap<ReportId, Report>, ReportError> {
        self.reports.find_all(ids).await.map_err(ReportError::from)
    }

    #[record_error_severity]
    #[tracing::instrument(name = "report.find_all_report_runs", skip(self), fields(count = ids.len()))]
    pub async fn find_all_report_runs(
        &self,
        ids: &[ReportRunId],
    ) -> Result<std::collections::HashMap<ReportRunId, ReportRun>, ReportError> {
        self.report_runs
            .find_all(ids)
            .await
            .map_err(ReportError::from)
    }

    #[record_error_severity]
    #[tracing::instrument(name = "report.list_report_runs", skip(self, query), fields(subject = %sub))]
    pub async fn list_report_runs(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<ReportRunsByCreatedAtCursor>,
    ) -> Result<es_entity::PaginatedQueryRet<ReportRun, ReportRunsByCreatedAtCursor>, ReportError>
    {
        self.authz
            .enforce_permission(
                sub,
                ReportObject::all_reports(),
                CoreReportAction::REPORT_READ,
            )
            .await?;
        Ok(self
            .report_runs
            .list_by_created_at(query, es_entity::ListDirection::Descending)
            .await?)
    }

    #[record_error_severity]
    #[tracing::instrument(name = "report.list_reports_for_run", skip(self), fields(subject = %sub, run_id = %run_id))]
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

        Ok(self
            .reports
            .list_for_run_id_by_created_at(
                run_id,
                Default::default(),
                es_entity::ListDirection::Descending,
            )
            .await?
            .entities)
    }

    #[record_error_severity]
    #[tracing::instrument(name = "report.find_report_run_by_id", skip(self), fields(subject = %sub, run_id = tracing::field::Empty))]
    pub async fn find_report_run_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<ReportRunId> + std::fmt::Debug,
    ) -> Result<Option<ReportRun>, ReportError> {
        let id = id.into();
        tracing::Span::current().record("run_id", id.to_string());

        self.authz
            .enforce_permission(
                sub,
                ReportObject::all_reports(),
                CoreReportAction::REPORT_READ,
            )
            .await?;

        Ok(self.report_runs.maybe_find_by_id(id).await?)
    }

    #[record_error_severity]
    #[tracing::instrument(name = "report.generate_report_file_download_link", skip(self), fields(subject = %sub, report_id = tracing::field::Empty, extension = %extension))]
    pub async fn generate_report_file_download_link(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        report_id: impl Into<ReportId> + std::fmt::Debug,
        extension: String,
    ) -> Result<String, ReportError> {
        let report_id = report_id.into();
        tracing::Span::current().record("report_id", report_id.to_string());

        self.authz
            .enforce_permission(
                sub,
                ReportObject::Report(AllOrOne::ById(report_id)),
                CoreReportAction::REPORT_READ,
            )
            .await?;

        let report = match self.reports.find_by_id(report_id).await {
            Ok(report) => report,
            Err(e) if e.was_not_found() => return Err(ReportError::NotFound),
            Err(e) => return Err(e.into()),
        };

        let file = match report.files.iter().find(|f| f.extension == extension) {
            Some(file) => file,
            None => return Err(ReportError::NotFound),
        };

        let location = cloud_storage::LocationInStorage {
            path: &file.path_in_bucket,
        };

        let download_link = self.storage.generate_download_link(location).await?;
        Ok(download_link)
    }

    #[record_error_severity]
    #[tracing::instrument(name = "report.reports_sync", skip(self), fields(job_id = tracing::field::Empty))]
    pub async fn reports_sync(&self) -> Result<job::JobId, ReportError> {
        let mut db = self.report_runs.begin_op().await?;
        let job_id = job::JobId::new();
        self.sync_reports_spawner
            .spawn_in_op(&mut db, job_id, SyncReportsJobConfig::<E>::new())
            .await?;

        tracing::Span::current().record("job_id", job_id.to_string());
        db.commit().await?;

        Ok(job_id)
    }

    #[record_error_severity]
    #[tracing::instrument(name = "report.trigger_report_run_job", skip(self), fields(subject = %sub, job_id = tracing::field::Empty))]
    pub async fn trigger_report_run_job(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<job::JobId, ReportError> {
        self.authz
            .enforce_permission(
                sub,
                ReportObject::all_reports(),
                CoreReportAction::REPORT_GENERATE,
            )
            .await?;

        let mut db = self.report_runs.begin_op().await?;
        let job_id = job::JobId::new();
        self.trigger_report_run_spawner
            .spawn_in_op(&mut db, job_id, TriggerReportRunJobConfig::<E>::new())
            .await?;
        tracing::Span::current().record("job_id", job_id.to_string());
        db.commit().await?;

        Ok(job_id)
    }
}
