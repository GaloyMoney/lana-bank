use es_entity::clock::ClockHandle;
use sqlx::PgPool;

pub use es_entity::Sort;
use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::{primitives::*, public::CoreCustomerEvent};

use super::{entity::*, error::*, publisher::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Prospect",
    err = "ProspectError",
    columns(
        email(ty = "String", list_by),
        telegram_id(ty = "String", list_by),
        public_id(ty = "PublicId", list_by),
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub struct ProspectRepo<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    pool: PgPool,
    publisher: ProspectPublisher<E>,
    clock: ClockHandle,
}

impl<E> Clone for ProspectRepo<E>
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

impl<E> ProspectRepo<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    pub(crate) fn new(pool: &PgPool, publisher: &ProspectPublisher<E>, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
            clock,
        }
    }

    async fn publish_in_op(
        &self,
        db: &mut impl es_entity::AtomicOperation,
        entity: &Prospect,
        new_events: es_entity::LastPersisted<'_, ProspectEvent>,
    ) -> Result<(), ProspectError> {
        self.publisher.publish_in_op(db, entity, new_events).await
    }
}

impl From<(ProspectsSortBy, &Prospect)> for prospect_cursor::ProspectsCursor {
    fn from(prospect_with_sort: (ProspectsSortBy, &Prospect)) -> Self {
        let (sort, prospect) = prospect_with_sort;
        match sort {
            ProspectsSortBy::CreatedAt => {
                prospect_cursor::ProspectsByCreatedAtCursor::from(prospect).into()
            }
            ProspectsSortBy::Email => {
                prospect_cursor::ProspectsByEmailCursor::from(prospect).into()
            }
            ProspectsSortBy::TelegramId => {
                prospect_cursor::ProspectsByTelegramIdCursor::from(prospect).into()
            }
            ProspectsSortBy::Id => prospect_cursor::ProspectsByIdCursor::from(prospect).into(),
            ProspectsSortBy::PublicId => {
                prospect_cursor::ProspectsByPublicIdCursor::from(prospect).into()
            }
        }
    }
}
