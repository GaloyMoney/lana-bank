use sqlx::PgPool;

use es_entity::*;
use outbox::OutboxEventMarker;

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
}

impl<E> RoleRepo<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    pub fn new(pool: &PgPool, publisher: &UserPublisher<E>) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
        }
    }

    async fn publish(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &Role,
        new_events: es_entity::LastPersisted<'_, RoleEvent>,
    ) -> Result<(), RoleError> {
        self.publisher.publish_role(db, entity, new_events).await
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
        }
    }
}
