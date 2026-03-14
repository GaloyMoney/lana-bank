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
    columns(
        party_id(ty = "PartyId", list_by),
        public_id(ty = "PublicId", list_by),
        stage(ty = "ProspectStage", list_for),
        customer_type(ty = "CustomerType", list_for, update(persist = false))
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

    fn cursor_to_id(cursor: prospect_cursor::ProspectsCursor) -> ProspectId {
        match cursor {
            prospect_cursor::ProspectsCursor::Byid(c) => c.id,
            prospect_cursor::ProspectsCursor::Bycreated_at(c) => c.id,
            prospect_cursor::ProspectsCursor::Bypublic_id(c) => c.id,
            prospect_cursor::ProspectsCursor::Byparty_id(c) => c.id,
        }
    }

    pub async fn list_by_party_email(
        &self,
        filter: &ProspectsFilters,
        direction: ListDirection,
        query: PaginatedQueryArgs<prospect_cursor::ProspectsCursor>,
    ) -> Result<(Vec<Prospect>, bool), ProspectError> {
        let first = query.first;
        let after_id = query.after.map(Self::cursor_to_id);
        let limit = first as i64 + 1;
        let stage = filter.stage.as_ref().map(|s| s.to_string());
        let customer_type = filter.customer_type.as_ref().map(|ct| ct.to_string());

        let ids = match direction {
            ListDirection::Ascending => {
                sqlx::query_scalar!(
                    r#"SELECT p.id AS "id: ProspectId"
                       FROM core_prospects p
                       JOIN core_parties pa ON pa.id = p.party_id
                       WHERE ($1::text IS NULL OR p.stage = $1)
                         AND ($2::text IS NULL OR p.customer_type = $2)
                         AND (
                           $3::uuid IS NULL
                           OR (pa.email, p.id) > (
                             (SELECT pa2.email FROM core_prospects p2 JOIN core_parties pa2 ON pa2.id = p2.party_id WHERE p2.id = $3),
                             $3
                           )
                         )
                       ORDER BY pa.email ASC, p.id ASC
                       LIMIT $4"#,
                    stage,
                    customer_type,
                    after_id as Option<ProspectId>,
                    limit,
                )
                .fetch_all(self.pool())
                .await?
            }
            ListDirection::Descending => {
                sqlx::query_scalar!(
                    r#"SELECT p.id AS "id: ProspectId"
                       FROM core_prospects p
                       JOIN core_parties pa ON pa.id = p.party_id
                       WHERE ($1::text IS NULL OR p.stage = $1)
                         AND ($2::text IS NULL OR p.customer_type = $2)
                         AND (
                           $3::uuid IS NULL
                           OR (pa.email, p.id) < (
                             (SELECT pa2.email FROM core_prospects p2 JOIN core_parties pa2 ON pa2.id = p2.party_id WHERE p2.id = $3),
                             $3
                           )
                         )
                       ORDER BY pa.email DESC, p.id DESC
                       LIMIT $4"#,
                    stage,
                    customer_type,
                    after_id as Option<ProspectId>,
                    limit,
                )
                .fetch_all(self.pool())
                .await?
            }
        };

        self.hydrate_ordered(ids, first).await
    }

    pub async fn list_by_party_telegram(
        &self,
        filter: &ProspectsFilters,
        direction: ListDirection,
        query: PaginatedQueryArgs<prospect_cursor::ProspectsCursor>,
    ) -> Result<(Vec<Prospect>, bool), ProspectError> {
        let first = query.first;
        let after_id = query.after.map(Self::cursor_to_id);
        let limit = first as i64 + 1;
        let stage = filter.stage.as_ref().map(|s| s.to_string());
        let customer_type = filter.customer_type.as_ref().map(|ct| ct.to_string());

        let ids = match direction {
            ListDirection::Ascending => {
                sqlx::query_scalar!(
                    r#"SELECT p.id AS "id: ProspectId"
                       FROM core_prospects p
                       JOIN core_parties pa ON pa.id = p.party_id
                       WHERE ($1::text IS NULL OR p.stage = $1)
                         AND ($2::text IS NULL OR p.customer_type = $2)
                         AND (
                           $3::uuid IS NULL
                           OR (pa.telegram_handle, p.id) > (
                             (SELECT pa2.telegram_handle FROM core_prospects p2 JOIN core_parties pa2 ON pa2.id = p2.party_id WHERE p2.id = $3),
                             $3
                           )
                         )
                       ORDER BY pa.telegram_handle ASC, p.id ASC
                       LIMIT $4"#,
                    stage,
                    customer_type,
                    after_id as Option<ProspectId>,
                    limit,
                )
                .fetch_all(self.pool())
                .await?
            }
            ListDirection::Descending => {
                sqlx::query_scalar!(
                    r#"SELECT p.id AS "id: ProspectId"
                       FROM core_prospects p
                       JOIN core_parties pa ON pa.id = p.party_id
                       WHERE ($1::text IS NULL OR p.stage = $1)
                         AND ($2::text IS NULL OR p.customer_type = $2)
                         AND (
                           $3::uuid IS NULL
                           OR (pa.telegram_handle, p.id) < (
                             (SELECT pa2.telegram_handle FROM core_prospects p2 JOIN core_parties pa2 ON pa2.id = p2.party_id WHERE p2.id = $3),
                             $3
                           )
                         )
                       ORDER BY pa.telegram_handle DESC, p.id DESC
                       LIMIT $4"#,
                    stage,
                    customer_type,
                    after_id as Option<ProspectId>,
                    limit,
                )
                .fetch_all(self.pool())
                .await?
            }
        };

        self.hydrate_ordered(ids, first).await
    }

    async fn hydrate_ordered(
        &self,
        ids: Vec<ProspectId>,
        first: usize,
    ) -> Result<(Vec<Prospect>, bool), ProspectError> {
        let has_next_page = ids.len() > first;
        let ids: Vec<ProspectId> = ids.into_iter().take(first).collect();

        let mut entities_map: std::collections::HashMap<ProspectId, Prospect> =
            self.find_all(&ids).await?;

        let ordered: Vec<Prospect> = ids
            .iter()
            .filter_map(|id| entities_map.remove(id))
            .collect();

        Ok((ordered, has_next_page))
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
