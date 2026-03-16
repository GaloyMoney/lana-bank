use tracing::{Span, instrument};

use core_time_events::CoreTimeEvent;
use job::JobType;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use super::collect_accounts_for_activity_classification::{
    CollectAccountsForActivityClassificationConfig,
    CollectAccountsForActivityClassificationJobSpawner,
};

pub const DEPOSIT_END_OF_DAY: JobType = JobType::new("outbox.deposit-end-of-day");

pub struct DepositEndOfDayHandler {
    collect_spawner: CollectAccountsForActivityClassificationJobSpawner,
}

impl DepositEndOfDayHandler {
    pub fn new(collect_spawner: CollectAccountsForActivityClassificationJobSpawner) -> Self {
        Self { collect_spawner }
    }
}

impl<E> OutboxEventHandler<E> for DepositEndOfDayHandler
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    #[instrument(name = "deposit_sync.deposit_end_of_day.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreTimeEvent::EndOfDay { closing_time, .. }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.collect_spawner
                .spawn_in_op(
                    op,
                    job::JobId::new(),
                    CollectAccountsForActivityClassificationConfig {
                        closing_time: *closing_time,
                    },
                )
                .await?;
        }
        Ok(())
    }
}
