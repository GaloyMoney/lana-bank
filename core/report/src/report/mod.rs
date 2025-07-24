mod entity;
pub mod error;
mod repo;

use audit::AuditSvc;
use authz::PermissionCheck;
use outbox::{Outbox, OutboxEventMarker};

use crate::{event::CoreReportEvent, primitives::*, publisher::ReportPublisher};

pub use entity::{NewReport, Report, ReportFile};
pub use repo::report_cursor::ReportsByCreatedAtCursor;

#[cfg(feature = "json-schema")]
pub use entity::ReportEvent;
#[cfg(not(feature = "json-schema"))]
pub(crate) use entity::ReportEvent;

pub use error::*;
use repo::*;

pub struct Reports<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    authz: Perms,
    repo: ReportRepo<E>,
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
        authz: Perms,
        publisher: &ReportPublisher<E>,
        _outbox: &Outbox<E>,
    ) -> Result<Self, ReportError> {
        let repo = ReportRepo::new(pool, publisher);

        Ok(Self { repo, authz })
    }

    pub(super) fn repo(&self) -> &ReportRepo<E> {
        &self.repo
    }

    pub async fn find_all(
        &self,
        ids: &[ReportId],
    ) -> Result<std::collections::HashMap<ReportId, Report>, ReportError> {
        self.repo.find_all(ids).await
    }

    pub async fn list_for_run_id(&self, run_id: ReportRunId) -> Result<Vec<Report>, ReportError> {
        let reports = self
            .repo
            .list_for_run_id_by_created_at(
                run_id,
                Default::default(),
                es_entity::ListDirection::Descending,
            )
            .await?;
        Ok(reports.entities)
    }

    pub async fn find_by_id(&self, id: ReportId) -> Result<Report, ReportError> {
        self.repo.find_by_id(id).await
    }
}
