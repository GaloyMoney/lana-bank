use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::primitives::*;

use crate::{event::CoreCreditEvent, publisher::CreditFacilityPublisher};

use super::{entity::*, error::PaymentError};

#[derive(EsRepo)]
#[es_repo(
    entity = "Payment",
    err = "PaymentError",
    columns(credit_facility_id(ty = "CreditFacilityId", list_for, update(persist = false)),),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub struct PaymentRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[allow(dead_code)]
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
    clock: ClockHandle,
}

impl<E> Clone for PaymentRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
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
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(pool: &PgPool, publisher: &CreditFacilityPublisher<E>, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
            clock,
        }
    }

    #[record_error_severity]
    #[tracing::instrument(name = "payment.publish_in_op", skip_all)]
    async fn publish_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Payment,
        new_events: es_entity::LastPersisted<'_, PaymentEvent>,
    ) -> Result<(), PaymentError> {
        self.publisher
            .publish_payment_in_op(op, entity, new_events)
            .await
    }
}
