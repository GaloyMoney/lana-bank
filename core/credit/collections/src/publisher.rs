use obix::out::{Outbox, OutboxEventMarker};

use crate::event::CoreCollectionsEvent;

pub struct CollectionsPublisher<E>
where
    E: OutboxEventMarker<CoreCollectionsEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for CollectionsPublisher<E>
where
    E: OutboxEventMarker<CoreCollectionsEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> CollectionsPublisher<E>
where
    E: OutboxEventMarker<CoreCollectionsEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    pub async fn publish_all(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        events: Vec<CoreCollectionsEvent>,
    ) -> Result<(), impl std::error::Error> {
        self.outbox.publish_all_persisted(op, events).await
    }
}
