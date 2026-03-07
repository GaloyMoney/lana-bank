use es_entity::clock::ClockHandle;
use sqlx::{PgPool, Row, types::Uuid};

use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::{primitives::*, public::CoreAccessEvent, publisher::UserPublisher};

use super::{cursor::UserCursor, entity::*, error::*};

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

    pub async fn list_users(
        &self,
        args: PaginatedQueryArgs<UserCursor>,
    ) -> Result<PaginatedQueryRet<User, UserCursor>, UserError> {
        let first = args.first as i64;
        let cursor_id = args.after.map(|c| sqlx::types::Uuid::from(c.id));

        let rows = sqlx::query(
            r#"SELECT id FROM core_users
               WHERE ($2::UUID IS NULL OR id > $2)
               ORDER BY id ASC
               LIMIT $1"#,
        )
        .bind(first + 1)
        .bind(cursor_id)
        .fetch_all(&self.pool)
        .await?;

        let ids: Vec<UserId> = rows
            .into_iter()
            .map(|r| {
                let id: Uuid = r.get("id");
                UserId::from(id)
            })
            .collect();

        let has_next_page = ids.len() > args.first;
        let ids_to_load = if has_next_page {
            &ids[..args.first]
        } else {
            &ids
        };

        let mut users_map = self.find_all(ids_to_load).await?;

        let mut entities = Vec::new();
        for id in ids_to_load {
            if let Some(user) = users_map.remove(id) {
                entities.push(user);
            }
        }

        let end_cursor = entities.last().map(UserCursor::from);

        Ok(PaginatedQueryRet {
            entities,
            has_next_page,
            end_cursor,
        })
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
