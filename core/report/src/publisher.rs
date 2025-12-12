use crate::report::{Report, ReportError, ReportEvent};
use crate::report_run::{ReportRun, ReportRunError, ReportRunEvent};

use super::event::CoreReportEvent;
use outbox::{Outbox, OutboxEventMarker};

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

    #[allow(dead_code)]
    pub async fn publish_report(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Report,
        new_events: es_entity::LastPersisted<'_, ReportEvent>,
    ) -> Result<(), ReportError> {
        use ReportEvent::*;
        let publish_events = new_events
            .map(|event| match &event.event {
                Initialized { .. } => CoreReportEvent::ReportCreated { id: entity.id },
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn publish_report_run(
        &self,
        db: &mut impl es_entity::AtomicOperation,
        entity: &ReportRun,
        new_events: es_entity::LastPersisted<'_, ReportRunEvent>,
    ) -> Result<(), ReportRunError> {
        use ReportRunEvent::*;
        let publish_events = new_events
            .map(|event| match &event.event {
                Initialized { .. } => CoreReportEvent::ReportRunCreated { id: entity.id },
                StateUpdated { .. } => CoreReportEvent::ReportRunStateUpdated { id: entity.id },
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(db, publish_events)
            .await?;
        Ok(())
    }
}
