use crate::report_run::{ReportRun, ReportRunError, ReportRunEvent};
use crate::{CoreReportEvent, PublicReportRun};
use obix::out::{Outbox, OutboxEventMarker};

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
    pub async fn publish_report_run(
        &self,
        db: &mut impl es_entity::AtomicOperation,
        entity: &ReportRun,
        new_events: es_entity::LastPersisted<'_, ReportRunEvent>,
    ) -> Result<(), ReportRunError> {
        use ReportRunEvent::*;
        let publish_events = new_events
            .map(|event| match &event.event {
                Initialized { .. } => CoreReportEvent::ReportRunCreated {
                    entity: PublicReportRun::from(entity),
                },
                StateUpdated { .. } => CoreReportEvent::ReportRunStateUpdated {
                    entity: PublicReportRun::from(entity),
                },
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(db, publish_events)
            .await?;
        Ok(())
    }
}
