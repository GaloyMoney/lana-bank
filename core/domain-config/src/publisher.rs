use obix::out::{Outbox, OutboxEventMarker};

use crate::{DomainConfig, DomainConfigError, DomainConfigEvent, public::CoreDomainConfigEvent};

pub struct DomainConfigPublisher<E>
where
    E: OutboxEventMarker<CoreDomainConfigEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for DomainConfigPublisher<E>
where
    E: OutboxEventMarker<CoreDomainConfigEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> DomainConfigPublisher<E>
where
    E: OutboxEventMarker<CoreDomainConfigEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    pub async fn publish(
        &self,
        db: &mut impl es_entity::AtomicOperation,
        entity: &DomainConfig,
        new_events: es_entity::LastPersisted<'_, DomainConfigEvent>,
    ) -> Result<(), DomainConfigError> {
        use DomainConfigEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Updated { .. } => Some(CoreDomainConfigEvent::DomainConfigUpdated {
                    key: entity.key.clone(),
                }),
                Initialized { .. } => None,
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(db, publish_events)
            .await?;
        Ok(())
    }
}
