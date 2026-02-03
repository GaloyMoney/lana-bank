use crate::report_run::{ReportRun, ReportRunError, ReportRunEvent};
use crate::{CoreReportEvent, PublicReportRun, REPORT_RUN_EVENT_TYPE};
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
    pub async fn publish_report_run_in_op(
        &self,
        db: &mut impl es_entity::AtomicOperation,
        entity: &ReportRun,
        new_events: es_entity::LastPersisted<'_, ReportRunEvent>,
    ) -> Result<(), ReportRunError> {
        use ReportRunEvent::*;
        for event in new_events {
            let publish_event = match &event.event {
                Initialized { .. } => CoreReportEvent::ReportRunCreated {
                    entity: PublicReportRun::from(entity),
                },
                StateUpdated { .. } => CoreReportEvent::ReportRunStateUpdated {
                    entity: PublicReportRun::from(entity),
                },
            };
            self.outbox
                .publish_ephemeral_in_op(db, REPORT_RUN_EVENT_TYPE, publish_event)
                .await?;
        }
        Ok(())
    }
}
