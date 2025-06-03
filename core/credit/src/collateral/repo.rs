use sqlx::PgPool;

use es_entity::*;
use outbox::OutboxEventMarker;

use crate::{CreditFacilityPublisher, event::CoreCreditEvent, primitives::CollateralId};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Collateral",
    err = "CollateralError",
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub struct CollateralRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
}

impl<E> CollateralRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(pool: &PgPool, publisher: &CreditFacilityPublisher<E>) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
        }
    }

    async fn publish(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &Collateral,
        new_events: es_entity::LastPersisted<'_, CollateralEvent>,
    ) -> Result<(), CollateralError> {
        self.publisher
            .publish_collateral(db, entity, new_events)
            .await
    }
}

impl<E> Clone for CollateralRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            publisher: self.publisher.clone(),
        }
    }
}
