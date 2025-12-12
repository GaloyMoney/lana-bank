use sqlx::PgPool;

use es_entity::*;
use outbox::OutboxEventMarker;

use crate::{event::CoreReportEvent, primitives::*, publisher::ReportPublisher};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Report",
    err = "ReportError",
    columns(external_id(ty = "String"), run_id(ty = "ReportRunId", list_for)),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub(crate) struct ReportRepo<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    #[allow(dead_code)]
    pool: PgPool,
    publisher: ReportPublisher<E>,
}

impl<E> ReportRepo<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new(pool: &PgPool, publisher: &ReportPublisher<E>) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
        }
    }

    #[allow(dead_code)]
    async fn publish(
        &self,
        db: &mut impl es_entity::AtomicOperation,
        entity: &Report,
        new_events: es_entity::LastPersisted<'_, ReportEvent>,
    ) -> Result<(), ReportError> {
        self.publisher.publish_report(db, entity, new_events).await
    }
}

impl<E> Clone for ReportRepo<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    fn clone(&self) -> Self {
        Self {
            publisher: self.publisher.clone(),
            pool: self.pool.clone(),
        }
    }
}
