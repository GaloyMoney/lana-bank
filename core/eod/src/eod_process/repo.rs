use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::primitives::*;
use crate::public::CoreEodEvent;
use crate::publisher::EodPublisher;

use super::entity::*;

#[derive(EsRepo)]
#[es_repo(
    entity = "EodProcess",
    columns(
        date(ty = "chrono::NaiveDate", list_by),
        status(
            ty = "EodProcessStatus",
            create(accessor = "status()"),
            update(accessor = "status()")
        ),
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub(crate) struct EodProcessRepo<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    publisher: EodPublisher<E>,
    pool: PgPool,
    clock: ClockHandle,
}

impl<E> Clone for EodProcessRepo<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    fn clone(&self) -> Self {
        Self {
            publisher: self.publisher.clone(),
            pool: self.pool.clone(),
            clock: self.clock.clone(),
        }
    }
}

impl<E> EodProcessRepo<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    pub fn new(pool: &PgPool, publisher: &EodPublisher<E>, clock: ClockHandle) -> Self {
        Self {
            publisher: publisher.clone(),
            pool: pool.clone(),
            clock,
        }
    }

    async fn publish_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &EodProcess,
        new_events: es_entity::LastPersisted<'_, EodProcessEvent>,
    ) -> Result<(), sqlx::Error> {
        self.publisher
            .publish_eod_process_in_op(op, entity, new_events)
            .await
    }
}
