use sqlx::PgPool;

use es_entity::*;
use outbox::OutboxEventMarker;

use crate::{event::CoreCreditEvent, primitives::*, publisher::*};

use super::{entity::*, error::CreditFacilityProposalError};

#[derive(EsRepo)]
#[es_repo(
    id = "CreditFacilityId",
    entity = "CreditFacilityProposal",
    err = "CreditFacilityProposalError",
    columns(
        customer_id(ty = "CustomerId", list_for, update(persist = false)),
        approval_process_id(ty = "ApprovalProcessId", list_by, update(persist = "false")),
        collateral_id(ty = "CollateralId", update(persist = false)),
    ),
    tbl_prefix = "core",
    // post_persist_hook = "publish"
)]
pub struct CreditFacilityProposalRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
}

impl<E> Clone for CreditFacilityProposalRepo<E>
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

impl<E> CreditFacilityProposalRepo<E>
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
    //     op: &mut impl es_entity::AtomicOperation,
    //     entity: &CreditFacilityProposal,
    //     new_events: es_entity::LastPersisted<'_, CreditFacilityProposalEvent>,
    // ) -> Result<(), CreditFacilityProposalError> {
    //     self.publisher
    //         .publish_proposal(op, entity, new_events)
    //         .await
    // }
}
