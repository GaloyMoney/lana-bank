use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::{
    primitives::{ApprovalProcessId, CalaTransactionId, DepositAccountId, PublicId, WithdrawalId},
    public::CoreDepositEvent,
    publisher::DepositPublisher,
};

use super::entity::*;

#[derive(EsRepo)]
#[es_repo(
    entity = "Withdrawal",
    columns(
        deposit_account_id(
            ty = "DepositAccountId",
            list_for(by(created_at)),
            update(persist = false)
        ),
        approval_process_id(ty = "ApprovalProcessId", update(persist = false)),
        cancelled_tx_id(ty = "Option<CalaTransactionId>", create(persist = false)),
        reference(ty = "String", create(accessor = "reference()")),
        public_id(ty = "PublicId", list_by),
        status(ty = "WithdrawalStatus", list_for, update(accessor = "status()"))
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub struct WithdrawalRepo<E>
where
    E: OutboxEventMarker<CoreDepositEvent>,
{
    publisher: DepositPublisher<E>,
    pool: PgPool,
    clock: ClockHandle,
}

impl<E> Clone for WithdrawalRepo<E>
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

impl<E> WithdrawalRepo<E>
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
        entity: &Withdrawal,
        new_events: es_entity::LastPersisted<'_, WithdrawalEvent>,
    ) -> Result<(), sqlx::Error> {
        self.publisher
            .publish_withdrawal_in_op(op, entity, new_events)
            .await
    }
}

impl From<(WithdrawalsSortBy, &Withdrawal)> for withdrawal_cursor::WithdrawalsCursor {
    fn from(withdrawal_with_sort: (WithdrawalsSortBy, &Withdrawal)) -> Self {
        let (sort, withdrawal) = withdrawal_with_sort;
        match sort {
            WithdrawalsSortBy::CreatedAt => {
                withdrawal_cursor::WithdrawalsByCreatedAtCursor::from(withdrawal).into()
            }
            WithdrawalsSortBy::Id => {
                withdrawal_cursor::WithdrawalsByIdCursor::from(withdrawal).into()
            }
            WithdrawalsSortBy::PublicId => {
                withdrawal_cursor::WithdrawalsByPublicIdCursor::from(withdrawal).into()
            }
        }
    }
}

mod withdrawal_status_sqlx {
    use sqlx::{Type, postgres::*};

    use super::WithdrawalStatus;

    impl Type<Postgres> for WithdrawalStatus {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for WithdrawalStatus {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <String as sqlx::Encode<'_, Postgres>>::encode(self.to_string(), buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for WithdrawalStatus {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(s.parse().map_err(|e: strum::ParseError| Box::new(e))?)
        }
    }

    impl PgHasArrayType for WithdrawalStatus {
        fn array_type_info() -> PgTypeInfo {
            <String as sqlx::postgres::PgHasArrayType>::array_type_info()
        }
    }
}
