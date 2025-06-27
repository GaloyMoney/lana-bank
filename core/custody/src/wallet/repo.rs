use sqlx::PgPool;

use es_entity::*;
use outbox::OutboxEventMarker;

use crate::primitives::WalletId;
use crate::{event::CoreCustodyEvent, publisher::CustodyPublisher};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Wallet",
    err = "WalletError",
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub struct WalletRepo<E>
where
    E: OutboxEventMarker<CoreCustodyEvent>,
{
    pool: PgPool,
    publisher: CustodyPublisher<E>,
}

impl<E> WalletRepo<E>
where
    E: OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(pool: &PgPool, publisher: &CustodyPublisher<E>) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
        }
    }

    async fn publish(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &Wallet,
        new_events: es_entity::LastPersisted<'_, WalletEvent>,
    ) -> Result<(), WalletError> {
        self.publisher.publish_wallet(db, entity, new_events).await
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
        }
    }
}
