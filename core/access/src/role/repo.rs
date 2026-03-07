use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::{primitives::*, public::CoreAccessEvent, publisher::UserPublisher};

use super::entity::*;

#[derive(EsRepo)]
#[es_repo(
    entity = "Role",
    columns(name(ty = "String", list_by)),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
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

    async fn publish_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Role,
        new_events: es_entity::LastPersisted<'_, RoleEvent>,
    ) -> Result<(), sqlx::Error> {
        self.publisher
            .publish_role_in_op(op, entity, new_events)
            .await
    }
}

impl From<(RolesSortBy, &Role)> for role_cursor::RolesCursor {
    fn from(role_with_sort: (RolesSortBy, &Role)) -> Self {
        let (sort, role) = role_with_sort;
        match sort {
            RolesSortBy::CreatedAt => role_cursor::RolesByCreatedAtCursor::from(role).into(),
            RolesSortBy::Id => role_cursor::RolesByIdCursor::from(role).into(),
            RolesSortBy::Name => role_cursor::RolesByNameCursor::from(role).into(),
        }
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
