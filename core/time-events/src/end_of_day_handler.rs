use tracing::{Span, instrument};

use crate::CoreTimeEvent;
use job::{JobSpec, JobType};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use crate::{
    eod_process::{EodProcesses, NewEodProcess, error::EodProcessError},
    job_id::{self, eod_process_id_from_date},
    process_manager::EodProcessManagerJobSpawner,
    public::CoreEodEvent,
};

pub const EOD_END_OF_DAY: JobType = JobType::new("outbox.eod-end-of-day");

pub struct EndOfDayHandler<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    pm_spawner: EodProcessManagerJobSpawner,
    eod_processes: EodProcesses<E>,
}

impl<E> EndOfDayHandler<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    pub fn new(pm_spawner: EodProcessManagerJobSpawner, eod_processes: EodProcesses<E>) -> Self {
        Self {
            pm_spawner,
            eod_processes,
        }
    }
}

impl<E, Ev> OutboxEventHandler<Ev> for EndOfDayHandler<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
    Ev: OutboxEventMarker<CoreTimeEvent>,
{
    #[instrument(name = "eod.end_of_day.process_message", parent = None, skip(self, op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<Ev>,
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

            // Deterministic process ID from date
            let process_id = eod_process_id_from_date(day);

            // Create the EodProcess entity (idempotent via duplicate key)
            let new_process = NewEodProcess::builder().id(process_id).date(*day).build()?;

            match self.eod_processes.create_in_op(op, new_process).await {
                Ok(_) => {}
                Err(EodProcessError::Create(ref e)) if e.was_duplicate() => {
                    // Already created — idempotent
                }
                Err(e) => return Err(e.into()),
            }

            // Spawn the PM job
            let job_id = job_id::eod_manager_id(day);
            let spec = JobSpec::new(
                job_id,
                crate::process_manager::EodProcessManagerConfig {
                    date: *day,
                    closing_time: *closing_time,
                    process_id,
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
