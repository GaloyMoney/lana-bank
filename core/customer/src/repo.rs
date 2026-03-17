use es_entity::clock::ClockHandle;
use sqlx::PgPool;

pub use es_entity::Sort;
use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::primitives::*;
use crate::{error::CustomerError, public::CoreCustomerEvent, publisher::*};

use super::entity::*;

#[derive(EsRepo)]
#[es_repo(
    entity = "Customer",
    columns(
        party_id(ty = "PartyId", list_by),
        customer_type(ty = "CustomerType", list_for, update(persist = false)),
        public_id(ty = "PublicId", list_by),
        status(ty = "CustomerStatus", list_for)
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub struct CustomerRepo<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    pool: PgPool,
    publisher: CustomerPublisher<E>,
    clock: ClockHandle,
}

impl<E> Clone for CustomerRepo<E>
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

impl<E> CustomerRepo<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    pub(super) fn new(pool: &PgPool, publisher: &CustomerPublisher<E>, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
            clock,
        }
    }

    async fn publish_in_op(
        &self,
        db: &mut impl es_entity::AtomicOperation,
        entity: &Customer,
        new_events: es_entity::LastPersisted<'_, CustomerEvent>,
    ) -> Result<(), sqlx::Error> {
        self.publisher.publish_in_op(db, entity, new_events).await
    }

    fn cursor_to_id(cursor: customer_cursor::CustomersCursor) -> CustomerId {
        match cursor {
            customer_cursor::CustomersCursor::Byid(c) => c.id,
            customer_cursor::CustomersCursor::Bycreated_at(c) => c.id,
            customer_cursor::CustomersCursor::Bypublic_id(c) => c.id,
            customer_cursor::CustomersCursor::Byparty_id(c) => c.id,
        }
    }

    pub async fn list_by_party_email(
        &self,
        filter: &CustomersFilters,
        direction: ListDirection,
        query: PaginatedQueryArgs<customer_cursor::CustomersCursor>,
    ) -> Result<(Vec<Customer>, bool), CustomerError> {
        let first = query.first;
        let after_id = query.after.map(Self::cursor_to_id);
        let limit = first as i64 + 1;
        let status = filter.status.as_ref().map(|s| s.to_string());
        let customer_type = filter.customer_type.as_ref().map(|ct| ct.to_string());

        let ids = match direction {
            ListDirection::Ascending => {
                sqlx::query_scalar!(
                    r#"SELECT c.id AS "id: CustomerId"
                       FROM core_customers c
                       JOIN core_parties pa ON pa.id = c.party_id
                       WHERE ($1::text IS NULL OR c.status = $1)
                         AND ($2::text IS NULL OR c.customer_type = $2)
                         AND (
                           $3::uuid IS NULL
                           OR (pa.email, c.id) > (
                             (SELECT pa2.email FROM core_customers c2 JOIN core_parties pa2 ON pa2.id = c2.party_id WHERE c2.id = $3),
                             $3
                           )
                         )
                       ORDER BY pa.email ASC, c.id ASC
                       LIMIT $4"#,
                    status,
                    customer_type,
                    after_id as Option<CustomerId>,
                    limit,
                )
                .fetch_all(self.pool())
                .await?
            }
            ListDirection::Descending => {
                sqlx::query_scalar!(
                    r#"SELECT c.id AS "id: CustomerId"
                       FROM core_customers c
                       JOIN core_parties pa ON pa.id = c.party_id
                       WHERE ($1::text IS NULL OR c.status = $1)
                         AND ($2::text IS NULL OR c.customer_type = $2)
                         AND (
                           $3::uuid IS NULL
                           OR (pa.email, c.id) < (
                             (SELECT pa2.email FROM core_customers c2 JOIN core_parties pa2 ON pa2.id = c2.party_id WHERE c2.id = $3),
                             $3
                           )
                         )
                       ORDER BY pa.email DESC, c.id DESC
                       LIMIT $4"#,
                    status,
                    customer_type,
                    after_id as Option<CustomerId>,
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
        filter: &CustomersFilters,
        direction: ListDirection,
        query: PaginatedQueryArgs<customer_cursor::CustomersCursor>,
    ) -> Result<(Vec<Customer>, bool), CustomerError> {
        let first = query.first;
        let after_id = query.after.map(Self::cursor_to_id);
        let limit = first as i64 + 1;
        let status = filter.status.as_ref().map(|s| s.to_string());
        let customer_type = filter.customer_type.as_ref().map(|ct| ct.to_string());

        let ids = match direction {
            ListDirection::Ascending => {
                sqlx::query_scalar!(
                    r#"SELECT c.id AS "id: CustomerId"
                       FROM core_customers c
                       JOIN core_parties pa ON pa.id = c.party_id
                       WHERE ($1::text IS NULL OR c.status = $1)
                         AND ($2::text IS NULL OR c.customer_type = $2)
                         AND (
                           $3::uuid IS NULL
                           OR (pa.telegram_handle, c.id) > (
                             (SELECT pa2.telegram_handle FROM core_customers c2 JOIN core_parties pa2 ON pa2.id = c2.party_id WHERE c2.id = $3),
                             $3
                           )
                         )
                       ORDER BY pa.telegram_handle ASC, c.id ASC
                       LIMIT $4"#,
                    status,
                    customer_type,
                    after_id as Option<CustomerId>,
                    limit,
                )
                .fetch_all(self.pool())
                .await?
            }
            ListDirection::Descending => {
                sqlx::query_scalar!(
                    r#"SELECT c.id AS "id: CustomerId"
                       FROM core_customers c
                       JOIN core_parties pa ON pa.id = c.party_id
                       WHERE ($1::text IS NULL OR c.status = $1)
                         AND ($2::text IS NULL OR c.customer_type = $2)
                         AND (
                           $3::uuid IS NULL
                           OR (pa.telegram_handle, c.id) < (
                             (SELECT pa2.telegram_handle FROM core_customers c2 JOIN core_parties pa2 ON pa2.id = c2.party_id WHERE c2.id = $3),
                             $3
                           )
                         )
                       ORDER BY pa.telegram_handle DESC, c.id DESC
                       LIMIT $4"#,
                    status,
                    customer_type,
                    after_id as Option<CustomerId>,
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
        ids: Vec<CustomerId>,
        first: usize,
    ) -> Result<(Vec<Customer>, bool), CustomerError> {
        let has_next_page = ids.len() > first;
        let ids: Vec<CustomerId> = ids.into_iter().take(first).collect();

        let mut entities_map: std::collections::HashMap<CustomerId, Customer> =
            self.find_all(&ids).await?;

        let ordered: Vec<Customer> = ids
            .iter()
            .filter_map(|id| entities_map.remove(id))
            .collect();

        Ok((ordered, has_next_page))
    }
}

mod customer_status_sqlx {
    use sqlx::{Type, postgres::*};

    use crate::primitives::CustomerStatus;

    impl Type<Postgres> for CustomerStatus {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for CustomerStatus {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <String as sqlx::Encode<'_, Postgres>>::encode(self.to_string(), buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for CustomerStatus {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(s.parse().map_err(|e: strum::ParseError| Box::new(e))?)
        }
    }

    impl PgHasArrayType for CustomerStatus {
        fn array_type_info() -> PgTypeInfo {
            <String as sqlx::postgres::PgHasArrayType>::array_type_info()
        }
    }
}

mod customer_type_sqlx {
    use sqlx::{Type, postgres::*};

    use crate::primitives::CustomerType;

    impl Type<Postgres> for CustomerType {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for CustomerType {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <String as sqlx::Encode<'_, Postgres>>::encode(self.to_string(), buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for CustomerType {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(s.parse().map_err(|e: strum::ParseError| Box::new(e))?)
        }
    }

    impl PgHasArrayType for CustomerType {
        fn array_type_info() -> PgTypeInfo {
            <String as sqlx::postgres::PgHasArrayType>::array_type_info()
        }
    }
}

impl From<(CustomersSortBy, &Customer)> for customer_cursor::CustomersCursor {
    fn from(customer_with_sort: (CustomersSortBy, &Customer)) -> Self {
        let (sort, customer) = customer_with_sort;
        match sort {
            CustomersSortBy::CreatedAt => {
                customer_cursor::CustomersByCreatedAtCursor::from(customer).into()
            }
            CustomersSortBy::Id => customer_cursor::CustomersByIdCursor::from(customer).into(),
            CustomersSortBy::PublicId => {
                customer_cursor::CustomersByPublicIdCursor::from(customer).into()
            }
            CustomersSortBy::PartyId => {
                customer_cursor::CustomersByPartyIdCursor::from(customer).into()
            }
        }
    }
}
