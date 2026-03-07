use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::{primitives::*, public::CoreAccessEvent, publisher::UserPublisher};

use super::entity::*;

#[derive(EsRepo)]
#[es_repo(
    entity = "User",
    columns(email(ty = "String", list_by),),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub(crate) struct UserRepo<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    #[allow(dead_code)]
    pool: PgPool,
    publisher: UserPublisher<E>,
    clock: ClockHandle,
}

impl<E> UserRepo<E>
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
        entity: &User,
        new_events: es_entity::LastPersisted<'_, UserEvent>,
    ) -> Result<(), sqlx::Error> {
        self.publisher
            .publish_user_in_op(op, entity, new_events)
            .await
    }
}

impl From<(UsersSortBy, &User)> for user_cursor::UsersCursor {
    fn from(user_with_sort: (UsersSortBy, &User)) -> Self {
        let (sort, user) = user_with_sort;
        match sort {
            UsersSortBy::CreatedAt => user_cursor::UsersByCreatedAtCursor::from(user).into(),
            UsersSortBy::Id => user_cursor::UsersByIdCursor::from(user).into(),
            UsersSortBy::Email => user_cursor::UsersByEmailCursor::from(user).into(),
        }
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
            clock: self.clock.clone(),
        }
    }
}
