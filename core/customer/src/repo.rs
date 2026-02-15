use es_entity::clock::ClockHandle;
use sqlx::PgPool;

pub use es_entity::Sort;
use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::{primitives::*, public::CoreCustomerEvent, publisher::*};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Customer",
    err = "CustomerError",
    columns(
        email(ty = "String", list_by),
        telegram_handle(ty = "String", list_by),
        kyc_verification(ty = "KycVerification", list_for),
        activity(ty = "Activity", list_for),
        public_id(ty = "PublicId", list_by)
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
    ) -> Result<(), CustomerError> {
        self.publisher.publish_in_op(db, entity, new_events).await
    }
}

mod account_status_sqlx {
    use sqlx::{Type, postgres::*};

    use crate::primitives::KycVerification;

    impl Type<Postgres> for KycVerification {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for KycVerification {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <String as sqlx::Encode<'_, Postgres>>::encode(self.to_string(), buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for KycVerification {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
            match s.as_str() {
                "pending-verification" | "no-kyc" => Ok(KycVerification::NoKyc),
                _ => Ok(s.parse().map_err(|e: strum::ParseError| Box::new(e))?),
            }
        }
    }

    impl PgHasArrayType for KycVerification {
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
            CustomersSortBy::TelegramHandle => {
                customer_cursor::CustomersByTelegramHandleCursor::from(customer).into()
            }
            CustomersSortBy::Id => customer_cursor::CustomersByIdCursor::from(customer).into(),
            CustomersSortBy::PublicId => {
                customer_cursor::CustomersByPublicIdCursor::from(customer).into()
            }
        }
    }
}
