use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::primitives::{CustodianId, WalletId};
use crate::{CoreCustodyEvent, publisher::CustodyPublisher};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Wallet",
    columns(external_wallet_id(ty = "str", find_by)),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
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

    async fn publish_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Wallet,
        new_events: es_entity::LastPersisted<'_, WalletEvent>,
    ) -> Result<(), sqlx::Error> {
        self.publisher
            .publish_wallet_in_op(op, entity, new_events)
            .await
    }

    pub async fn list_all_by_custodian_id_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        custodian_id: CustodianId,
    ) -> Result<Vec<Wallet>, WalletError> {
        let wallet_ids = sqlx::query_scalar::<_, uuid::Uuid>(
            r#"
            SELECT id
            FROM core_wallet_events
            WHERE event_type = 'initialized'
              AND (event->>'custodian_id')::uuid = $1
            ORDER BY recorded_at ASC
            "#,
        )
        .bind(custodian_id)
        .fetch_all(op.as_executor())
        .await?;

        let mut wallets = Vec::with_capacity(wallet_ids.len());
        for wallet_id in wallet_ids {
            wallets.push(
                self.find_by_id_in_op(&mut *op, WalletId::from(wallet_id))
                    .await?,
            );
        }

        Ok(wallets)
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
