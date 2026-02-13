use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{CoreCreditEvent, primitives::*, publisher::*};

use super::{entity::*, error::PendingCreditFacilityError};

#[derive(EsRepo)]
#[es_repo(
    entity = "PendingCreditFacility",
    err = "PendingCreditFacilityError",
    columns(
        customer_id(ty = "CustomerId", list_for, update(persist = false)),
        credit_facility_proposal_id(ty = "CreditFacilityProposalId", update(persist = false)),
        approval_process_id(ty = "ApprovalProcessId", list_by, update(persist = "false")),
        collateral_id(ty = "CollateralId", update(persist = false)),
        collateralization_ratio(
            ty = "CollateralizationRatio",
            list_by,
            create(persist = false),
            update(accessor = "last_collateralization_ratio()")
        ),
        collateralization_state(
            ty = "PendingCreditFacilityCollateralizationState",
            list_for,
            update(accessor = "last_collateralization_state()")
        ),
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub struct PendingCreditFacilityRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
    clock: ClockHandle,
}

impl<E> Clone for PendingCreditFacilityRepo<E>
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

impl<E> PendingCreditFacilityRepo<E>
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
    #[tracing::instrument(name = "pending_credit_facility.publish_in_op", skip_all)]
    async fn publish_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &PendingCreditFacility,
        new_events: es_entity::LastPersisted<'_, PendingCreditFacilityEvent>,
    ) -> Result<(), PendingCreditFacilityError> {
        self.publisher
            .publish_pending_credit_facility_in_op(op, entity, new_events)
            .await
    }
}
