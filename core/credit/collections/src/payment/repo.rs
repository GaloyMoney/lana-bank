use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::primitives::*;

use crate::{event::CoreCollectionsEvent, publisher::CollectionsPublisher};

use super::{entity::*, error::PaymentError};

#[derive(EsRepo)]
#[es_repo(
    entity = "Payment",
    err = "PaymentError",
    columns(facility_id(ty = "FacilityId", list_for, update(persist = false)),),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub(crate) struct PaymentRepo<E>
where
    E: OutboxEventMarker<CoreCollectionsEvent>,
{
    #[allow(dead_code)]
    pool: PgPool,
    publisher: CollectionsPublisher<E>,
    clock: ClockHandle,
}

impl<E> Clone for PaymentRepo<E>
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

impl<E> PaymentRepo<E>
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
    #[tracing::instrument(name = "payment.publish", skip_all)]
    async fn publish(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Payment,
        new_events: es_entity::LastPersisted<'_, PaymentEvent>,
    ) -> Result<(), PaymentError> {
        // TODO: Implement publish_payment method in CollectionsPublisher
        Ok(())
    }
}
