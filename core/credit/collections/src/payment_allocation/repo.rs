use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::primitives::*;

use crate::{event::CoreCollectionsEvent, publisher::CollectionsPublisher};

use super::{entity::*, error::PaymentAllocationError};

#[derive(EsRepo)]
#[es_repo(
    entity = "PaymentAllocation",
    err = "PaymentAllocationError",
    columns(
        facility_id(ty = "FacilityId", list_for, update(persist = false)),
        payment_id(ty = "PaymentId", list_for, update(persist = false)),
        obligation_id(ty = "ObligationId", update(persist = false)),
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub(crate) struct PaymentAllocationRepo<E>
where
    E: OutboxEventMarker<CoreCollectionsEvent>,
{
    #[allow(dead_code)]
    pool: PgPool,
    publisher: CollectionsPublisher<E>,
    clock: ClockHandle,
}

impl<E> Clone for PaymentAllocationRepo<E>
where
    E: OutboxEventMarker<CoreCollectionsEvent>,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            publisher: self.publisher.clone(),
            clock: self.clock.clone(),
        }
    }
}
impl<E> PaymentAllocationRepo<E>
where
    E: OutboxEventMarker<CoreCollectionsEvent>,
{
    pub fn new(pool: &PgPool, publisher: &CollectionsPublisher<E>, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
            clock,
        }
    }

    #[record_error_severity]
    #[tracing::instrument(name = "payment_allocation.publish", skip_all)]
    async fn publish(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &PaymentAllocation,
        new_events: es_entity::LastPersisted<'_, PaymentAllocationEvent>,
    ) -> Result<(), PaymentAllocationError> {
        // TODO: Implement publish_payment_allocation method in CollectionsPublisher
        Ok(())
    }
}
