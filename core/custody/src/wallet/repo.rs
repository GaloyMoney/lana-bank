use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::primitives::WalletId;
use crate::{event::CoreCustodyEvent, publisher::CustodyPublisher};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Wallet",
    err = "WalletError",
    columns(external_wallet_id(ty = "str", find_by)),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub struct WalletRepo<E>
where
    E: OutboxEventMarker<CoreCustodyEvent>,
{
    pool: PgPool,
    publisher: CustodyPublisher<E>,
    clock: ClockHandle,
}

impl<E> WalletRepo<E>
where
    E: OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(pool: &PgPool, publisher: &CustodyPublisher<E>, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
            clock,
        }
    }

    async fn publish(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Wallet,
        new_events: es_entity::LastPersisted<'_, WalletEvent>,
    ) -> Result<(), WalletError> {
        self.publisher.publish_wallet(op, entity, new_events).await
    }
}

impl<E> Clone for WalletRepo<E>
where
    E: OutboxEventMarker<CoreCustodyEvent>,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            publisher: self.publisher.clone(),
            clock: self.clock.clone(),
        }
    }
}
