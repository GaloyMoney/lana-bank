use es_entity::LastPersisted;
use outbox::{Outbox, OutboxEventMarker};

use crate::{entity::DomainConfigurationEvent, error::DomainConfigurationError, DomainConfiguration};

#[derive(Clone)]
pub struct DomainConfigurationPublisher<E>
where
    E: OutboxEventMarker<DomainConfigurationEvent>,
{
    outbox: Outbox<E>,
}

impl<E> DomainConfigurationPublisher<E>
where
    E: OutboxEventMarker<DomainConfigurationEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    pub(super) async fn publish(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        _entity: &DomainConfiguration,
        new_events: LastPersisted<'_, DomainConfigurationEvent>,
    ) -> Result<(), DomainConfigurationError> {
        let publish_events = new_events.map(|evt| evt.event.clone()).collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }
}
