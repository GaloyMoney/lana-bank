use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::{event::CoreAccessEvent, primitives::*, publisher::UserPublisher};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Role",
    err = "RoleError",
    columns(name(ty = "String", list_by)),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub(crate) struct RoleRepo<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    pool: PgPool,
    publisher: UserPublisher<E>,
    clock: ClockHandle,
}

impl<E> RoleRepo<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    pub fn new(pool: &PgPool, publisher: &UserPublisher<E>, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
            clock,
        }
    }

    async fn publish(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Role,
        new_events: es_entity::LastPersisted<'_, RoleEvent>,
    ) -> Result<(), RoleError> {
        self.publisher.publish_role(op, entity, new_events).await
    }
}

impl<E> Clone for RoleRepo<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    fn clone(&self) -> Self {
        Self {
            publisher: self.publisher.clone(),
            pool: self.pool.clone(),
            clock: self.clock.clone(),
        }
    }
}
