use sqlx::PgPool;

use es_entity::*;
use outbox::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{event::CoreCreditEvent, primitives::*, publisher::CreditFacilityPublisher};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "LiquidationProcess",
    err = "LiquidationProcessError",
    columns(
        obligation_id(ty = "ObligationId", list_for, update(persist = false)),
        credit_facility_id(ty = "CreditFacilityId", list_for, update(persist = false)),
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub struct LiquidationProcessRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
}

impl<E> Clone for LiquidationProcessRepo<E>
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

impl<E> LiquidationProcessRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(pool: &PgPool, publisher: &CreditFacilityPublisher<E>) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
        }
    }

    #[record_error_severity]
    #[tracing::instrument(name = "liquidation_process.publish", skip_all)]
    async fn publish(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &LiquidationProcess,
        new_events: es_entity::LastPersisted<'_, LiquidationProcessEvent>,
    ) -> Result<(), LiquidationProcessError> {
        self.publisher
            .publish_liquidation_process(op, entity, new_events)
            .await
    }
}
