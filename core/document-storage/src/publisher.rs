use obix::out::{Outbox, OutboxEventMarker};

use crate::entity::{Document, DocumentEvent, DocumentStatus};
use crate::error::DocumentStorageError;
use crate::event::CoreDocumentStorageEvent;

pub struct DocumentPublisher<E>
where
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for DocumentPublisher<E>
where
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> DocumentPublisher<E>
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
        op: &mut impl es_entity::AtomicOperation,
        entity: &Document,
        new_events: es_entity::LastPersisted<'_, DocumentEvent>,
    ) -> Result<(), DocumentStorageError> {
        use DocumentEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                FileUploaded { .. } => Some(CoreDocumentStorageEvent::DocumentStatusChanged {
                    document_id: entity.id,
                    reference_id: entity.reference_id,
                    status: DocumentStatus::Active,
                    recorded_at: event.recorded_at,
                }),
                UploadFailed { .. } => Some(CoreDocumentStorageEvent::DocumentStatusChanged {
                    document_id: entity.id,
                    reference_id: entity.reference_id,
                    status: DocumentStatus::Failed,
                    recorded_at: event.recorded_at,
                }),
                Archived { .. } => Some(CoreDocumentStorageEvent::DocumentStatusChanged {
                    document_id: entity.id,
                    reference_id: entity.reference_id,
                    status: DocumentStatus::Archived,
                    recorded_at: event.recorded_at,
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
