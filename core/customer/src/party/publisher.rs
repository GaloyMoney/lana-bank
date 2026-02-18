use obix::out::{Outbox, OutboxEventMarker};

use super::{entity::*, error::*};
use crate::public::*;

pub struct PartyPublisher<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for PartyPublisher<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> PartyPublisher<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    pub async fn publish_in_op(
        &self,
        db: &mut impl es_entity::AtomicOperation,
        entity: &Party,
        new_events: es_entity::LastPersisted<'_, PartyEvent>,
    ) -> Result<(), PartyError> {
        use PartyEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Initialized { .. } => Some(CoreCustomerEvent::PartyCreated {
                    entity: PublicParty::from(entity),
                }),
                EmailUpdated { .. } => Some(CoreCustomerEvent::PartyEmailUpdated {
                    entity: PublicParty::from(entity),
                }),
                _ => None,
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(db, publish_events)
            .await?;
        Ok(())
    }
}
