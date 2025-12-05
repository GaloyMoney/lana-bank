use sqlx::PgPool;

use es_entity::*;
use outbox::OutboxEventMarker;

use crate::{
    error::CoreCreditFacilityError,
    event::CoreCreditFacilityEvent,
    primitives::{CollateralId, CustodyWalletId},
    publisher::CreditFacilityPublisher,
};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Collateral",
    err = "CoreCreditFacilityError",
    columns(custody_wallet_id(ty = "Option<CustodyWalletId>", update(persist = false))),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub struct CollateralRepo<E>
where
    E: OutboxEventMarker<CoreCreditFacilityEvent>,
{
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
}

impl<E> CollateralRepo<E>
where
    E: OutboxEventMarker<CoreCreditFacilityEvent>,
{
    pub fn new(pool: &PgPool, publisher: &CreditFacilityPublisher<E>) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
        }
    }

    #[tracing::instrument(name = "collateral.publish", skip_all, err(level = "warn"))]
    async fn publish(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Collateral,
        new_events: es_entity::LastPersisted<'_, CollateralEvent>,
    ) -> Result<(), CoreCreditFacilityError> {
        self.publisher
            .publish_collateral(op, entity, new_events)
            .await
    }
}

impl<E> Clone for CollateralRepo<E>
where
    E: OutboxEventMarker<CoreCreditFacilityEvent>,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            publisher: self.publisher.clone(),
        }
    }
}
