use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::primitives::*;

use crate::{public::CoreCreditCollectionEvent, publisher::CollectionPublisher};

use super::{entity::*, error::PaymentAllocationError};

#[derive(EsRepo)]
#[es_repo(
    entity = "PaymentAllocation",
    err = "PaymentAllocationError",
    columns(
        beneficiary_id(ty = "BeneficiaryId", list_for, update(persist = false)),
        payment_id(ty = "PaymentId", list_for, update(persist = false)),
        obligation_id(ty = "ObligationId", update(persist = false)),
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub(crate) struct PaymentAllocationRepo<E>
where
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    #[allow(dead_code)]
    pool: PgPool,
    publisher: CollectionPublisher<E>,
    clock: ClockHandle,
}

impl<E> Clone for PaymentAllocationRepo<E>
where
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
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
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub(crate) fn new(
        pool: &PgPool,
        publisher: &CollectionPublisher<E>,
        clock: ClockHandle,
    ) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
            clock,
        }
    }

    #[record_error_severity]
    #[tracing::instrument(name = "payment_allocation.publish_in_op", skip_all)]
    async fn publish_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &PaymentAllocation,
        new_events: es_entity::LastPersisted<'_, PaymentAllocationEvent>,
    ) -> Result<(), PaymentAllocationError> {
        self.publisher
            .publish_payment_allocation_in_op(op, entity, new_events)
            .await
    }
}
