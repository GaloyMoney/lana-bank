use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};
use tracing::{Span, instrument};

use job::JobSpec;

use crate::{
    primitives::EodProcessId,
    process_manager::{EodProcessManagerConfig, EodProcessManagerJobSpawner},
};

pub const END_OF_DAY_HANDLER_JOB: job::JobType = job::JobType::new("outbox.eod-end-of-day");

pub struct EndOfDayHandler {
    pm_spawner: EodProcessManagerJobSpawner,
    phase_names: Vec<String>,
}

impl EndOfDayHandler {
    pub fn new(pm_spawner: &EodProcessManagerJobSpawner, phase_names: Vec<String>) -> Self {
        Self {
            pm_spawner: pm_spawner.clone(),
            phase_names,
        }
    }
}

impl<E> OutboxEventHandler<E> for EndOfDayHandler
where
    E: OutboxEventMarker<core_time_events::CoreTimeEvent>,
{
    #[instrument(name = "eod.end_of_day_handler.process_message", parent = None, skip(self, op, event), fields(seq = %event.sequence, handled = false))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(core_time_events::CoreTimeEvent::EndOfDay {
            day,
            closing_time,
            timezone: _,
        }) = event.as_event::<core_time_events::CoreTimeEvent>()
        {
            event.inject_trace_parent();
            Span::current().record("handled", true);

            let process_id = EodProcessId::new();
            let manager_job_id = job::JobId::new();

            let spec = JobSpec::new(
                manager_job_id,
                EodProcessManagerConfig {
                    date: *day,
                    closing_time: *closing_time,
                    process_id,
                    phase_names: self.phase_names.clone(),
                },
            )
            .queue_id("eod-manager".to_string());
            self.pm_spawner.spawn_all_in_op(op, vec![spec]).await?;

            tracing::info!(
                day = %day,
                closing_time = %closing_time,
                process_id = %process_id,
                "EOD process manager job spawned from EndOfDay event"
            );
        }
        Ok(())
    }
}
