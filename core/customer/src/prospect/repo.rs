use es_entity::clock::ClockHandle;
use sqlx::PgPool;

pub use es_entity::Sort;
use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::{primitives::*, public::CoreCustomerEvent};

use super::{entity::*, publisher::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Prospect",
    columns(
        party_id(ty = "PartyId", list_by),
        public_id(ty = "PublicId", list_by),
        stage(ty = "ProspectStage", list_for)
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
    ) -> Result<(), sqlx::Error> {
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
            ProspectsSortBy::Id => prospect_cursor::ProspectsByIdCursor::from(prospect).into(),
            ProspectsSortBy::PublicId => {
                prospect_cursor::ProspectsByPublicIdCursor::from(prospect).into()
            }
            ProspectsSortBy::PartyId => {
                prospect_cursor::ProspectsByPartyIdCursor::from(prospect).into()
            }
        }
    }
}

mod prospect_stage_sqlx {
    use sqlx::{Type, postgres::*};

    use crate::primitives::ProspectStage;

    impl Type<Postgres> for ProspectStage {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for ProspectStage {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <String as sqlx::Encode<'_, Postgres>>::encode(self.to_string(), buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for ProspectStage {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(s.parse().map_err(|e: strum::ParseError| Box::new(e))?)
        }
    }

    impl PgHasArrayType for ProspectStage {
        fn array_type_info() -> PgTypeInfo {
            <String as sqlx::postgres::PgHasArrayType>::array_type_info()
        }
    }
}
