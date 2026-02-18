use es_entity::clock::ClockHandle;
use sqlx::PgPool;

pub use es_entity::Sort;
use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::{primitives::*, public::CoreCustomerEvent};

use super::{entity::*, error::*, publisher::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Party",
    err = "PartyError",
    columns(email(ty = "String", list_by), telegram_handle(ty = "String", list_by),),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub struct PartyRepo<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    pool: PgPool,
    publisher: PartyPublisher<E>,
    clock: ClockHandle,
}

impl<E> Clone for PartyRepo<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            publisher: self.publisher.clone(),
            clock: self.clock.clone(),
        }
    }
}

impl<E> PartyRepo<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    pub(crate) fn new(pool: &PgPool, publisher: &PartyPublisher<E>, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
            clock,
        }
    }

    async fn publish_in_op(
        &self,
        db: &mut impl es_entity::AtomicOperation,
        entity: &Party,
        new_events: es_entity::LastPersisted<'_, PartyEvent>,
    ) -> Result<(), PartyError> {
        self.publisher.publish_in_op(db, entity, new_events).await
    }
}
