use sqlx::PgPool;

use es_entity::*;
use outbox::OutboxEventMarker;

use crate::{event::CoreCreditEvent, primitives::*, publisher::*};

use super::{entity::*, error::CreditFacilityProposalError};

#[derive(EsRepo)]
#[es_repo(
    entity = "CreditFacilityProposal",
    err = "CreditFacilityProposalError",
    columns(
        customer_id(ty = "CustomerId", list_for, update(persist = false)),
        approval_process_id(ty = "ApprovalProcessId", list_by, update(persist = "false")),
        collateral_id(ty = "CollateralId", update(persist = false)),
        collateralization_ratio(
            ty = "CollateralizationRatio",
            list_by,
            create(persist = false),
            update(accessor = "last_collateralization_ratio()")
        ),
        collateralization_state(
            ty = "CreditFacilityProposalCollateralizationState",
            list_for,
            update(accessor = "last_collateralization_state()")
        ),
    ),
    tbl_prefix = "core"
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
}
