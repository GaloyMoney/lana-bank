use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::{
    primitives::{
        Activity, DepositAccountHolderId, DepositAccountId, DepositAccountStatus, PublicId,
    },
    public::CoreDepositEvent,
    publisher::DepositPublisher,
};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "DepositAccount",
    columns(
        account_holder_id(
            ty = "DepositAccountHolderId",
            list_for(by(created_at, id)),
            update(persist = false)
        ),
        activity(ty = "Activity", list_for),
        public_id(ty = "PublicId", list_by),
        status(ty = "DepositAccountStatus", list_for, update(accessor = "status"))
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub struct DepositAccountRepo<E>
where
    E: OutboxEventMarker<CoreDepositEvent>,
{
    publisher: DepositPublisher<E>,
    pool: PgPool,
    clock: ClockHandle,
}

impl<E> Clone for DepositAccountRepo<E>
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

impl<E> DepositAccountRepo<E>
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
        entity: &DepositAccount,
        new_events: es_entity::LastPersisted<'_, DepositAccountEvent>,
    ) -> Result<(), sqlx::Error> {
        self.publisher
            .publish_deposit_account_in_op(op, entity, new_events)
            .await
    }

    pub async fn list_all(&self) -> Result<Vec<DepositAccount>, DepositAccountError> {
        let mut accounts = Vec::new();
        let mut next = Some(PaginatedQueryArgs::<
            deposit_account_cursor::DepositAccountsByIdCursor,
        >::default());

        while let Some(query) = next.take() {
            let mut ret = self
                .list_by_id(query, es_entity::ListDirection::Descending)
                .await?;
            accounts.append(&mut ret.entities);
            next = ret.into_next_query();
        }

        Ok(accounts)
    }
}

impl From<(DepositAccountsSortBy, &DepositAccount)>
    for deposit_account_cursor::DepositAccountsCursor
{
    fn from(account_with_sort: (DepositAccountsSortBy, &DepositAccount)) -> Self {
        let (sort, account) = account_with_sort;
        match sort {
            DepositAccountsSortBy::CreatedAt => {
                deposit_account_cursor::DepositAccountsByCreatedAtCursor::from(account).into()
            }
            DepositAccountsSortBy::Id => {
                deposit_account_cursor::DepositAccountsByIdCursor::from(account).into()
            }
            DepositAccountsSortBy::PublicId => {
                deposit_account_cursor::DepositAccountsByPublicIdCursor::from(account).into()
            }
        }
    }
}

mod deposit_account_status_sqlx {
    use sqlx::{Type, postgres::*};

    use crate::primitives::DepositAccountStatus;

    impl Type<Postgres> for DepositAccountStatus {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for DepositAccountStatus {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <String as sqlx::Encode<'_, Postgres>>::encode(self.to_string(), buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for DepositAccountStatus {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(s.parse().map_err(|e: strum::ParseError| Box::new(e))?)
        }
    }

    impl PgHasArrayType for DepositAccountStatus {
        fn array_type_info() -> PgTypeInfo {
            <String as sqlx::postgres::PgHasArrayType>::array_type_info()
        }
    }
}
