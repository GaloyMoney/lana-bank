use tracing::{Span, instrument};

use core_time_events::CoreTimeEvent;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use super::update_customer_activity_status_command::UpdateCustomerActivityStatusConfig;

pub const UPDATE_CUSTOMER_ACTIVITY_STATUS: JobType =
    JobType::new("outbox.update-customer-activity-status");

pub struct UpdateCustomerActivityStatusHandler {
    update_customer_activity_status: JobSpawner<UpdateCustomerActivityStatusConfig>,
}

impl UpdateCustomerActivityStatusHandler {
    pub fn new(
        update_customer_activity_status: JobSpawner<UpdateCustomerActivityStatusConfig>,
    ) -> Self {
        Self {
            update_customer_activity_status,
        }
    }
}

impl<E> OutboxEventHandler<E> for UpdateCustomerActivityStatusHandler
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    #[instrument(name = "customer_sync.update_customer_activity_status.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreTimeEvent::EndOfDay { closing_time, .. }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.update_customer_activity_status
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    UpdateCustomerActivityStatusConfig {
                        closing_time: *closing_time,
                    },
                    "end-of-day".to_string(),
                )
                .await?;
        }
        Ok(())
    }
}
