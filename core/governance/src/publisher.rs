use obix::out::{Outbox, OutboxEventMarker};

use crate::{
    approval_process::{ApprovalProcess, ApprovalProcessEvent},
    public::{GovernanceEvent, PublicApprovalProcess},
};

pub struct GovernancePublisher<E>
where
    E: OutboxEventMarker<GovernanceEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for GovernancePublisher<E>
where
    E: OutboxEventMarker<GovernanceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> GovernancePublisher<E>
where
    E: OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    pub async fn publish_approval_process_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &ApprovalProcess,
        new_events: es_entity::LastPersisted<'_, ApprovalProcessEvent>,
    ) -> Result<(), sqlx::Error> {
        use ApprovalProcessEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Concluded { .. } => Some(GovernanceEvent::ApprovalProcessConcluded {
                    entity: PublicApprovalProcess::from(entity),
                }),
                _ => None,
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }
}
