#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod airflow;
mod entity;
pub mod error;
mod event;
mod jobs;
mod primitives;
mod publisher;
mod repo;

use publisher::*;
use repo::*;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use job::Jobs;
use outbox::{Outbox, OutboxEventMarker};

use jobs::{SyncReportsJobConfig, SyncReportsJobInit};

pub use airflow::{
    AirflowConfig, DagRunStatusResponse, LastRun, ReportGenerateResponse, ReportsApiClient, RunType,
};
pub use entity::{Report, ReportEvent};
pub use error::ReportError;
pub use repo::report_cursor::ReportsByCreatedAtCursor;
pub use event::CoreReportEvent;
pub use primitives::*;

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::entity::ReportEvent;
}

pub struct Reports<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    authz: Perms,
    repo: ReportRepo<E>,
    airflow_client: ReportsApiClient,
}

impl<Perms, E> Clone for Reports<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            repo: self.repo.clone(),
            airflow_client: self.airflow_client.clone(),
        }
    }
}

impl<Perms, E> Reports<Perms, E>
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
    ) -> Result<Self, ReportError> {
        let publisher = ReportPublisher::new(outbox);
        let repo = ReportRepo::new(pool, &publisher);
        let airflow_client = ReportsApiClient::new(airflow_config.clone());

        jobs.add_initializer_and_spawn_unique(
            SyncReportsJobInit::new(airflow_client.clone(), repo.clone(), authz.clone()),
            SyncReportsJobConfig::new(),
        )
        .await?;

        Ok(Self {
            authz: authz.clone(),
            repo,
            airflow_client,
        })
    }

    #[instrument(name = "reports.generate_todays_report", skip(self), err)]
    pub async fn generate_todays_report(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<ReportGenerateResponse, ReportError> {
        self.authz
            .enforce_permission(
                sub,
                ReportObject::all_reports(),
                CoreReportAction::REPORT_GENERATE,
            )
            .await?;

        self.airflow_client.generate_todays_report().await
    }

    #[instrument(name = "reports.get_generation_status", skip(self), err)]
    pub async fn get_generation_status(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<DagRunStatusResponse, ReportError> {
        self.authz
            .enforce_permission(
                sub,
                ReportObject::all_reports(),
                CoreReportAction::REPORT_GENERATION_STATUS_READ,
            )
            .await?;

        self.airflow_client.get_generation_status().await
    }

    #[instrument(name = "reports.find_report_by_id", skip(self), err)]
    pub async fn find_report_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<ReportId> + std::fmt::Debug,
    ) -> Result<Option<Report>, ReportError> {
        self.authz
            .enforce_permission(
                sub,
                ReportObject::all_reports(),
                CoreReportAction::REPORT_READ,
            )
            .await?;

        match self.repo.find_by_id(id.into()).await {
            Ok(report) => Ok(Some(report)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(name = "reports.list_reports", skip(self), err)]
    pub async fn list_reports(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<ReportsByCreatedAtCursor>,
    ) -> Result<es_entity::PaginatedQueryRet<Report, ReportsByCreatedAtCursor>, ReportError> {
        self.authz
            .enforce_permission(
                sub,
                ReportObject::all_reports(),
                CoreReportAction::REPORT_READ,
            )
            .await?;

        Ok(self
            .repo
            .list_by_created_at(query, es_entity::ListDirection::Descending)
            .await?)
    }

    #[instrument(name = "reports.find_all_reports", skip(self), err)]
    pub async fn find_all_reports(
        &self,
        ids: &[ReportId],
    ) -> Result<std::collections::HashMap<ReportId, Report>, ReportError> {
        self.repo.find_all(ids).await
    }
}
