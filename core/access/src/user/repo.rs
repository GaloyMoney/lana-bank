use sqlx::PgPool;

use es_entity::*;
use outbox::OutboxEventMarker;

use crate::{event::CoreAccessEvent, primitives::*, publisher::UserPublisher};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "User",
    err = "UserError",
    columns(
        email(ty = "String", list_by),
        authentication_id(ty = "Option<AuthenticationId>", list_by, create(persist = false)),
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub(crate) struct UserRepo<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    #[allow(dead_code)]
    pool: PgPool,
    publisher: UserPublisher<E>,
}

impl<E> UserRepo<E>
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
        entity: &User,
        new_events: es_entity::LastPersisted<'_, UserEvent>,
    ) -> Result<(), UserError> {
        self.publisher.publish_user(db, entity, new_events).await
    }
}

impl<E> Clone for UserRepo<E>
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
