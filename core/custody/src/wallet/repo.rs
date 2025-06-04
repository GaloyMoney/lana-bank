use sqlx::PgPool;

use es_entity::*;
use outbox::OutboxEventMarker;

use crate::primitives::WalletId;
use crate::{event::CoreCustodyEvent, publisher::CustodyPublisher};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(entity = "Wallet", err = "WalletError", tbl_prefix = "core")]
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
