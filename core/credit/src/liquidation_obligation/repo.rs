use sqlx::PgPool;

use es_entity::*;
use outbox::OutboxEventMarker;

use crate::{
    event::CoreCreditEvent,
    primitives::{CreditFacilityId, ObligationId},
    publisher::CreditFacilityPublisher,
};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "LiquidationObligation",
    id = "ObligationId",
    err = "LiquidationObligationError",
    columns(
        parent_obligation_id(ty = "ObligationId", list_for, update(persist = false)),
        credit_facility_id(ty = "CreditFacilityId", list_for, update(persist = false)),
    ),
    tbl_prefix = "core",
    // post_persist_hook = "publish"
)]
pub struct LiquidationObligationRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
}

impl<E> Clone for LiquidationObligationRepo<E>
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

impl<E> LiquidationObligationRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(pool: &PgPool, publisher: &CreditFacilityPublisher<E>) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
        }
    }

    // async fn publish(
    //     &self,
    //     db: &mut es_entity::DbOp<'_>,
    //     entity: &LiquidationObligation,
    //     new_events: es_entity::LastPersisted<'_, LiquidationObligationEvent>,
    // ) -> Result<(), LiquidationObligationError> {
    //     self.publisher
    //         .publish_liquidation_obligation(db, entity, new_events)
    //         .await
    // }
}
