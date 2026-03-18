use obix::out::{Outbox, OutboxEventMarker};

use crate::{
    eod_process::{EodProcess, EodProcessEvent},
    public::{CoreEodEvent, PublicEodProcess},
};

pub struct EodPublisher<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for EodPublisher<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> EodPublisher<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    pub async fn publish_eod_process_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &EodProcess,
        new_events: es_entity::LastPersisted<'_, EodProcessEvent>,
    ) -> Result<(), sqlx::Error> {
        use EodProcessEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Initialized { .. } => Some(CoreEodEvent::EodProcessStarted {
                    entity: PublicEodProcess::from(entity),
                }),
                Completed { .. } => Some(CoreEodEvent::EodProcessCompleted {
                    entity: PublicEodProcess::from(entity),
                }),
                Failed { .. } => Some(CoreEodEvent::EodProcessFailed {
                    entity: PublicEodProcess::from(entity),
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
