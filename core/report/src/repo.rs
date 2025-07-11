use sqlx::PgPool;

use es_entity::*;
use outbox::OutboxEventMarker;

use crate::{entity::*, error::*, event::*, primitives::*, publisher::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Report",
    err = "ReportError",
    columns(
        path_in_bucket(ty = "String"),
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub struct ReportRepo<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pool: PgPool,
    publisher: ReportPublisher<E>,
}

impl<E> Clone for ReportRepo<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            publisher: self.publisher.clone(),
        }
    }
}

impl<E> ReportRepo<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub(super) fn new(pool: &PgPool, publisher: &ReportPublisher<E>) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
        }
    }

    async fn publish(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &Report,
        new_events: es_entity::LastPersisted<'_, ReportEvent>,
    ) -> Result<(), ReportError> {
        self.publisher.publish(db, entity, new_events).await
    }
}
