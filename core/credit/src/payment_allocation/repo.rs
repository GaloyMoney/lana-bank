use sqlx::PgPool;

use es_entity::*;
use outbox::OutboxEventMarker;

use crate::primitives::*;

use crate::{event::CoreCreditEvent, publisher::CreditFacilityPublisher};

use super::{entity::*, error::PaymentAllocationError};

#[derive(EsRepo)]
#[es_repo(
    entity = "PaymentAllocation",
    err = "PaymentAllocationError",
    columns(
        credit_facility_id(ty = "CreditFacilityId", list_for, update(persist = false)),
        payment_id(ty = "PaymentId", list_for, update(persist = false)),
        obligation_id(ty = "ObligationId", update(persist = false)),
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub struct PaymentAllocationRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[allow(dead_code)]
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
}

impl<E> Clone for PaymentAllocationRepo<E>
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
impl<E> PaymentAllocationRepo<E>
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
        entity: &PaymentAllocation,
        new_events: es_entity::LastPersisted<'_, PaymentAllocationEvent>,
    ) -> Result<(), PaymentAllocationError> {
        self.publisher
            .publish_payment_allocation(db, entity, new_events)
            .await
    }
}
