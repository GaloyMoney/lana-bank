#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod airflow;
pub mod entity;
pub mod error;
pub mod event;
pub mod primitives;
pub mod publisher;
pub mod repo;

use std::collections::HashMap;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use outbox::{Outbox, OutboxEventMarker};

pub use airflow::*;
pub use entity::*;
pub use error::*;
pub use event::*;
pub use primitives::*;
pub use publisher::ReportPublisher;
pub use repo::ReportRepo;

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
    ) -> Result<Self, ReportError> {
        let publisher = ReportPublisher::new(outbox);
        let repo = ReportRepo::new(pool, &publisher);
        let airflow_client = ReportsApiClient::new(airflow_config);

        let hz = airflow_client.health_check().await?;
        println!("====> Airflow health check: {}", hz.status);

        Ok(Self {
            authz: authz.clone(),
            repo,
            airflow_client,
        })
    }

    #[instrument(name = "reports.create_report", skip(self), err)]
    pub async fn create(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        name: impl Into<String> + std::fmt::Debug,
        date: chrono::DateTime<chrono::Utc>,
    ) -> Result<Report, ReportError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                ReportObject::all_reports(),
                CoreReportAction::REPORT_CREATE,
            )
            .await?;

        let new_report = NewReport::builder()
            .id(ReportId::new())
            .name(name.into())
            .date(date)
            .audit_info(audit_info)
            .build()
            .expect("Could not build report");

        self.repo.create(new_report).await
    }

    #[instrument(name = "reports.find_by_id", skip(self), err)]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<ReportId> + std::fmt::Debug,
    ) -> Result<Option<Report>, ReportError> {
        let id = id.into();
        self.authz
            .enforce_permission(sub, ReportObject::report(id), CoreReportAction::REPORT_READ)
            .await?;

        match self.repo.find_by_id(id).await {
            Ok(report) => Ok(Some(report)),
            Err(ReportError::EsEntityError(es_entity::EsEntityError::NotFound)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    #[instrument(name = "reports.find_all", skip(self), err)]
    pub async fn find_all<T: From<Report>>(
        &self,
        ids: &[ReportId],
    ) -> Result<HashMap<ReportId, T>, ReportError> {
        self.repo.find_all(ids).await
    }

    #[instrument(name = "reports.health_check", skip(self), err)]
    pub async fn health_check(&self) -> Result<HealthResponse, ReportError> {
        self.airflow_client.health_check().await
    }
}
