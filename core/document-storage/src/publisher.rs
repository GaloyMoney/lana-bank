use outbox::{Outbox, OutboxEventMarker};

use super::{entity::*, error::*, event::*};

pub struct DocumentStoragePublisher<E>
where
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for DocumentStoragePublisher<E>
where
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> DocumentStoragePublisher<E>
where
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    pub async fn publish(
        &self,
        db: &mut es_entity::DbOp<'_>,
        _entity: &Document,
        new_events: es_entity::LastPersisted<'_, DocumentEvent>,
    ) -> Result<(), DocumentStorageError> {
        use DocumentEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Initialized { id, .. } => {
                    Some(CoreDocumentStorageEvent::DocumentCreated { id: *id })
                }
                FileUploaded { .. } => None, // Don't publish file upload events for now
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(db.tx(), publish_events)
            .await?;
        Ok(())
    }
}
