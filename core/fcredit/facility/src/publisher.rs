use outbox::{Outbox, OutboxEventMarker};
use tracing::instrument;

use crate::{
    error::CoreCreditFacilityError,
    event::CoreCreditFacilityEvent,
    primitives::CreditFacilityProposalStatus,
    proposal::{CreditFacilityProposal, CreditFacilityProposalEvent},
};

pub struct CreditFacilityPublisher<E>
where
    E: OutboxEventMarker<CoreCreditFacilityEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for CreditFacilityPublisher<E>
where
    E: OutboxEventMarker<CoreCreditFacilityEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> CreditFacilityPublisher<E>
where
    E: OutboxEventMarker<CoreCreditFacilityEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    #[instrument(
        name = "credit.publisher.publish_proposal",
        skip_all,
        err(level = "warn")
    )]
    pub async fn publish_proposal(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &CreditFacilityProposal,
        new_events: es_entity::LastPersisted<'_, CreditFacilityProposalEvent>,
    ) -> Result<(), CoreCreditFacilityError> {
        use CreditFacilityProposalEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Initialized { amount, terms, .. } => {
                    Some(CoreCreditFacilityEvent::FacilityProposalCreated {
                        id: entity.id,
                        terms: *terms,
                        amount: *amount,
                        created_at: entity.created_at(),
                    })
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }
}
