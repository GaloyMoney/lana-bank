use outbox::{Outbox, OutboxEventMarker};

use crate::{entity::*, error::*, event::*};

pub struct ReportPublisher<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for ReportPublisher<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> ReportPublisher<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    pub async fn publish(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &Report,
        new_events: es_entity::LastPersisted<'_, ReportEvent>,
    ) -> Result<(), ReportError> {
        use ReportEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Initialized { .. } => Some(CoreReportEvent::ReportCreated {
                    id: entity.id,
                    name: entity.name.clone(),
                    date: entity.date,
                }),
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(db.tx(), publish_events)
            .await?;
        Ok(())
    }
}