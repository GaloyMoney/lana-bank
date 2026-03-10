use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::primitives::*;
use crate::public::GovernanceEvent;
use crate::publisher::GovernancePublisher;

use super::entity::*;

#[derive(EsRepo)]
#[es_repo(
    entity = "ApprovalProcess",
    columns(
        process_type(ty = "ApprovalProcessType"),
        committee_id(
            ty = "Option<CommitteeId>",
            list_for,
            create(accessor = "committee_id()"),
            update(accessor = "committee_id()")
        ),
        policy_id(ty = "PolicyId")
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub(crate) struct ApprovalProcessRepo<E>
where
    E: OutboxEventMarker<GovernanceEvent>,
{
    publisher: GovernancePublisher<E>,
    pool: PgPool,
    clock: ClockHandle,
}

impl<E> Clone for ApprovalProcessRepo<E>
where
    E: OutboxEventMarker<GovernanceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            publisher: self.publisher.clone(),
            pool: self.pool.clone(),
            clock: self.clock.clone(),
        }
    }
}

impl<E> ApprovalProcessRepo<E>
where
    E: OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(pool: &PgPool, publisher: &GovernancePublisher<E>, clock: ClockHandle) -> Self {
        Self {
            publisher: publisher.clone(),
            pool: pool.clone(),
            clock,
        }
    }

    async fn publish_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &ApprovalProcess,
        new_events: es_entity::LastPersisted<'_, ApprovalProcessEvent>,
    ) -> Result<(), sqlx::Error> {
        self.publisher
            .publish_approval_process_in_op(op, entity, new_events)
            .await
    }
}
