use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::{CoreReportEvent, primitives::*, publisher::ReportPublisher};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "ReportRun",
    err = "ReportRunError",
    columns(external_id(ty = "String")),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub struct ReportRunRepo<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    #[allow(dead_code)]
    pool: PgPool,
    publisher: ReportPublisher<E>,
}

impl<E> ReportRunRepo<E>
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
        op: &mut impl es_entity::AtomicOperation,
        entity: &ReportRun,
        new_events: es_entity::LastPersisted<'_, ReportRunEvent>,
    ) -> Result<(), ReportRunError> {
        self.publisher
            .publish_report_run(op, entity, new_events)
            .await
    }
}

impl<E> Clone for ReportRunRepo<E>
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
