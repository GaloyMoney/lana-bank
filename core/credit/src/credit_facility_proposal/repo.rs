use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{CoreCreditEvent, primitives::*, publisher::*};

use super::{entity::*, error::CreditFacilityProposalError};

#[derive(EsRepo)]
#[es_repo(
    entity = "CreditFacilityProposal",
    err = "CreditFacilityProposalError",
    columns(
        customer_id(ty = "CustomerId", list_for, update(persist = false)),
        approval_process_id(ty = "Option<ApprovalProcessId>", list_by, create(persist = "false")),
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub struct CreditFacilityProposalRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
    clock: ClockHandle,
}

impl<E> Clone for CreditFacilityProposalRepo<E>
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

impl<E> CreditFacilityProposalRepo<E>
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
    #[tracing::instrument(name = "credit_facility_proposal.publish_in_op", skip_all)]
    async fn publish_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &CreditFacilityProposal,
        new_events: es_entity::LastPersisted<'_, CreditFacilityProposalEvent>,
    ) -> Result<(), CreditFacilityProposalError> {
        self.publisher
            .publish_proposal_in_op(op, entity, new_events)
            .await
    }
}
