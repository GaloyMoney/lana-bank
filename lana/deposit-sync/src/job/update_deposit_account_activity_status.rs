use tracing::{Span, instrument};

use core_time_events::CoreTimeEvent;
use job::{JobId, JobSpawner, JobType};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use super::UpdateDepositAccountActivityStatusConfig;

pub const UPDATE_DEPOSIT_ACCOUNT_ACTIVITY_STATUS: JobType =
    JobType::new("outbox.update-deposit-account-activity-status");

pub struct UpdateDepositAccountActivityStatusHandler {
    execute_update: JobSpawner<UpdateDepositAccountActivityStatusConfig>,
}

impl UpdateDepositAccountActivityStatusHandler {
    pub fn new(execute_update: JobSpawner<UpdateDepositAccountActivityStatusConfig>) -> Self {
        Self { execute_update }
    }
}

impl<E> OutboxEventHandler<E> for UpdateDepositAccountActivityStatusHandler
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    #[instrument(name = "deposit_sync.update_deposit_account_activity_status.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreTimeEvent::EndOfDay { closing_time, .. }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.execute_update
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    UpdateDepositAccountActivityStatusConfig {
                        closing_time: *closing_time,
                    },
                    "deposit-activity-status".to_string(),
                )
                .await?;
        }
        Ok(())
    }
}
