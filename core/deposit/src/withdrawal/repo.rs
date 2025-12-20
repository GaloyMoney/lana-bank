use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::{
    event::CoreDepositEvent,
    primitives::{ApprovalProcessId, CalaTransactionId, DepositAccountId, PublicId, WithdrawalId},
    publisher::DepositPublisher,
};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Withdrawal",
    err = "WithdrawalError",
    columns(
        deposit_account_id(ty = "DepositAccountId", list_for, update(persist = false)),
        approval_process_id(ty = "ApprovalProcessId", update(persist = false)),
        cancelled_tx_id(ty = "Option<CalaTransactionId>", create(persist = false)),
        reference(ty = "String", create(accessor = "reference()")),
        public_id(ty = "PublicId", list_by)
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub struct WithdrawalRepo<E>
where
    E: OutboxEventMarker<CoreDepositEvent>,
{
    publisher: DepositPublisher<E>,

    pool: PgPool,
}

impl<E> Clone for WithdrawalRepo<E>
where
    E: OutboxEventMarker<CoreDepositEvent>,
{
    fn clone(&self) -> Self {
        Self {
            publisher: self.publisher.clone(),
            pool: self.pool.clone(),
        }
    }
}

impl<E> WithdrawalRepo<E>
where
    E: OutboxEventMarker<CoreDepositEvent>,
{
    pub fn new(pool: &PgPool, publisher: &DepositPublisher<E>) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
        }
    }

    async fn publish(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Withdrawal,
        new_events: es_entity::LastPersisted<'_, WithdrawalEvent>,
    ) -> Result<(), WithdrawalError> {
        self.publisher
            .publish_withdrawal(op, entity, new_events)
            .await
    }
}
