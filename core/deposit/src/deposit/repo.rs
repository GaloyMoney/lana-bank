use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::{
    primitives::{DepositAccountId, DepositId, PublicId},
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
        public_id(ty = "PublicId", list_by)
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
