use sqlx::PgPool;

pub use es_entity::Sort;
use es_entity::*;
use outbox::OutboxEventMarker;

use crate::{event::CoreCustomerEvent, primitives::*, publisher::*};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Customer",
    err = "CustomerError",
    columns(
        email(ty = "String", list_by),
        authentication_id(ty = "Option<AuthenticationId>", list_by, create(persist = false)),
        telegram_id(ty = "String", list_by),
        status(ty = "AccountStatus", list_for)
    ),
    post_persist_hook = "publish"
)]
pub struct CustomerRepo<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    pool: PgPool,
    publisher: CustomerPublisher<E>,
}

impl<E> Clone for CustomerRepo<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            publisher: self.publisher.clone(),
        }
    }
}

impl<E> CustomerRepo<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    pub(super) fn new(pool: &PgPool, publisher: &CustomerPublisher<E>) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
        }
    }

    async fn publish(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &Customer,
        new_events: es_entity::LastPersisted<'_, CustomerEvent>,
    ) -> Result<(), CustomerError> {
        self.publisher.publish(db, entity, new_events).await
    }
}

mod account_status_sqlx {
    use sqlx::{postgres::*, Type};

    use crate::primitives::AccountStatus;

    impl Type<Postgres> for AccountStatus {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for AccountStatus {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <String as sqlx::Encode<'_, Postgres>>::encode(self.to_string(), buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for AccountStatus {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(s.parse().map_err(|e: strum::ParseError| Box::new(e))?)
        }
    }

    impl PgHasArrayType for AccountStatus {
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
            CustomersSortBy::Email => {
                customer_cursor::CustomersByEmailCursor::from(customer).into()
            }
            CustomersSortBy::TelegramId => {
                customer_cursor::CustomersByTelegramIdCursor::from(customer).into()
            }
            CustomersSortBy::Id => customer_cursor::CustomersByIdCursor::from(customer).into(),
            CustomersSortBy::AuthenticationId => {
                customer_cursor::CustomersByAuthenticationIdCursor::from(customer).into()
            }
        }
    }
}
