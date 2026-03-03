use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;

use money::UsdCents;

use crate::{
    primitives::{DepositAccountId, DepositId, DepositStatus, PublicId},
    public::CoreDepositEvent,
    publisher::DepositPublisher,
};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Deposit",
    err = "DepositError",
    columns(
        deposit_account_id(
            ty = "DepositAccountId",
            list_for(by(created_at)),
            update(persist = false)
        ),
        reference(ty = "String", create(accessor = "reference()")),
        public_id(ty = "PublicId", list_by),
        status(ty = "DepositStatus", list_for, update(accessor = "status()")),
        amount(ty = "UsdCents", list_by, update(persist = false))
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub struct DepositRepo<E>
where
    E: OutboxEventMarker<CoreDepositEvent>,
{
    publisher: DepositPublisher<E>,
    pool: PgPool,
    clock: ClockHandle,
}

impl<E> Clone for DepositRepo<E>
where
    E: OutboxEventMarker<CoreDepositEvent>,
{
    fn clone(&self) -> Self {
        Self {
            publisher: self.publisher.clone(),
            pool: self.pool.clone(),
            clock: self.clock.clone(),
        }
    }
}

impl<E> DepositRepo<E>
where
    E: OutboxEventMarker<CoreDepositEvent>,
{
    pub fn new(pool: &PgPool, publisher: &DepositPublisher<E>, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
            clock,
        }
    }

    async fn publish_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Deposit,
        new_events: es_entity::LastPersisted<'_, DepositEvent>,
    ) -> Result<(), DepositError> {
        self.publisher
            .publish_deposit_in_op(op, entity, new_events)
            .await
    }
}

impl From<(DepositsSortBy, &Deposit)> for deposit_cursor::DepositsCursor {
    fn from(deposit_with_sort: (DepositsSortBy, &Deposit)) -> Self {
        let (sort, deposit) = deposit_with_sort;
        match sort {
            DepositsSortBy::CreatedAt => {
                deposit_cursor::DepositsByCreatedAtCursor::from(deposit).into()
            }
            DepositsSortBy::Id => deposit_cursor::DepositsByIdCursor::from(deposit).into(),
            DepositsSortBy::PublicId => {
                deposit_cursor::DepositsByPublicIdCursor::from(deposit).into()
            }
            DepositsSortBy::Amount => deposit_cursor::DepositsByAmountCursor::from(deposit).into(),
        }
    }
}

mod deposit_status_sqlx {
    use sqlx::{Type, postgres::*};

    use crate::primitives::DepositStatus;

    impl Type<Postgres> for DepositStatus {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for DepositStatus {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <String as sqlx::Encode<'_, Postgres>>::encode(self.to_string(), buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for DepositStatus {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(s.parse().map_err(|e: strum::ParseError| Box::new(e))?)
        }
    }

    impl PgHasArrayType for DepositStatus {
        fn array_type_info() -> PgTypeInfo {
            <String as sqlx::postgres::PgHasArrayType>::array_type_info()
        }
    }
}
