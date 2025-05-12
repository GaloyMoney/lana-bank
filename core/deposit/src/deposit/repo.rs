use sqlx::PgPool;

use es_entity::*;
use outbox::OutboxEventMarker;

use crate::{
    event::CoreDepositEvent,
    primitives::{DepositAccountId, DepositId},
    publisher::DepositPublisher,
};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Deposit",
    err = "DepositError",
    columns(
        deposit_account_id(ty = "DepositAccountId", list_for, update(persist = false)),
        reference(ty = "String", create(accessor = "reference()"))
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
        db: &mut es_entity::DbOp<'_>,
        entity: &Deposit,
        new_events: es_entity::LastPersisted<'_, DepositEvent>,
    ) -> Result<(), DepositError> {
        self.publisher.publish_deposit(db, entity, new_events).await
    }
}
