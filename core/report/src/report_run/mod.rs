mod entity;
pub mod error;
mod repo;

use audit::AuditSvc;
use authz::PermissionCheck;
use outbox::{Outbox, OutboxEventMarker};

use crate::{event::CoreReportEvent, primitives::*, publisher::ReportPublisher};

pub use entity::{NewReportRun, ReportRun, ReportRunState, ReportRunType};
pub use repo::report_run_cursor::ReportRunsByCreatedAtCursor;

#[cfg(feature = "json-schema")]
pub use entity::ReportRunEvent;
#[cfg(not(feature = "json-schema"))]
pub(crate) use entity::ReportRunEvent;

pub use error::*;
use repo::*;

pub struct ReportRuns<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreReportEvent>,
{
    authz: Perms,
    repo: ReportRunRepo<E>,
}

impl<Perms, E> Clone for ReportRuns<Perms, E>
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

impl<Perms, E> ReportRuns<Perms, E>
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
    ) -> Result<Self, ReportRunError> {
        let repo = ReportRunRepo::new(pool, publisher);

        Ok(Self { repo, authz })
    }

    pub(super) fn repo(&self) -> &ReportRunRepo<E> {
        &self.repo
    }

    pub async fn find_all(
        &self,
        ids: &[ReportRunId],
    ) -> Result<std::collections::HashMap<ReportRunId, ReportRun>, ReportRunError> {
        self.repo.find_all(ids).await
    }

    pub async fn list_by_created_at(
        &self,
        query: es_entity::PaginatedQueryArgs<ReportRunsByCreatedAtCursor>,
        direction: es_entity::ListDirection,
    ) -> Result<es_entity::PaginatedQueryRet<ReportRun, ReportRunsByCreatedAtCursor>, ReportRunError>
    {
        self.repo.list_by_created_at(query, direction).await
    }
}
