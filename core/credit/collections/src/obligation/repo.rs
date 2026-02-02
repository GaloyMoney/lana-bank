use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    event::CoreCreditCollectionsEvent,
    primitives::{BeneficiaryId, ObligationId},
    publisher::CollectionsPublisher,
};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Obligation",
    err = "ObligationError",
    columns(
        credit_facility_id(ty = "BeneficiaryId", list_for, update(persist = false)),
        reference(ty = "String", create(accessor = "reference()")),
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub struct ObligationRepo<E>
where
    E: OutboxEventMarker<CoreCreditCollectionsEvent>,
{
    pool: PgPool,
    publisher: CollectionsPublisher<E>,
    clock: ClockHandle,
}

impl<E> Clone for ObligationRepo<E>
where
    E: OutboxEventMarker<CoreCreditCollectionsEvent>,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            publisher: self.publisher.clone(),
            clock: self.clock.clone(),
        }
    }
}

impl<E> ObligationRepo<E>
where
    E: OutboxEventMarker<CoreCreditCollectionsEvent>,
{
    pub fn new(pool: &PgPool, publisher: &CollectionsPublisher<E>, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
            clock,
        }
    }

    #[record_error_severity]
    #[tracing::instrument(name = "obligation.publish", skip_all)]
    async fn publish(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Obligation,
        new_events: es_entity::LastPersisted<'_, ObligationEvent>,
    ) -> Result<(), ObligationError> {
        self.publisher
            .publish_obligation(op, entity, new_events)
            .await
    }
}
