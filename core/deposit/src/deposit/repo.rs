use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::{
    event::CoreDepositEvent,
    primitives::{DepositAccountId, DepositId, PublicId},
    publisher::DepositPublisher,
};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Deposit",
    err = "DepositError",
    columns(
        deposit_account_id(ty = "DepositAccountId", list_for, update(persist = false)),
        reference(ty = "String", create(accessor = "reference()")),
        public_id(ty = "PublicId", list_by)
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub struct DepositRepo<E>
where
    E: OutboxEventMarker<CoreDepositEvent>,
{
    publisher: DepositPublisher<E>,

    pool: PgPool,
}

impl<E> Clone for DepositRepo<E>
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

impl<E> DepositRepo<E>
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
        entity: &Deposit,
        new_events: es_entity::LastPersisted<'_, DepositEvent>,
    ) -> Result<(), DepositError> {
        self.publisher.publish_deposit(op, entity, new_events).await
    }
}
