use tracing::{instrument, Span};

use core_time_events::CoreTimeEvent;
use job::{JobSpec, JobType};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use crate::{job_id, process_manager::EodProcessManagerJobSpawner};

pub const EOD_END_OF_DAY: JobType = JobType::new("outbox.eod-end-of-day");

pub struct EndOfDayHandler {
    pm_spawner: EodProcessManagerJobSpawner,
}

impl EndOfDayHandler {
    pub fn new(pm_spawner: EodProcessManagerJobSpawner) -> Self {
        Self { pm_spawner }
    }
}

impl<E> OutboxEventHandler<E> for EndOfDayHandler
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    #[instrument(name = "eod.end_of_day.process_message", parent = None, skip(self, op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(
            e @ CoreTimeEvent::EndOfDay {
                day, closing_time, ..
            },
        ) = event.as_event()
        {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            let job_id = job_id::eod_manager_id(day);
            let spec = JobSpec::new(
                job_id,
                crate::process_manager::EodProcessManagerConfig {
                    date: *day,
                    closing_time: *closing_time,
                },
            )
            .queue_id("eod-manager".to_string());
            match self.pm_spawner.spawn_all_in_op(op, vec![spec]).await {
                Ok(_) | Err(job::error::JobError::DuplicateId(_)) => {}
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }
}
