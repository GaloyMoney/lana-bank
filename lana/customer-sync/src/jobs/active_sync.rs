use tracing::{Span, instrument};

use core_customer::CoreCustomerEvent;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use super::active_sync_job::CustomerActiveSyncConfig;

pub const CUSTOMER_ACTIVE_SYNC: JobType = JobType::new("outbox.customer-active-sync");

pub struct CustomerActiveSyncHandler {
    spawner: JobSpawner<CustomerActiveSyncConfig>,
}

impl CustomerActiveSyncHandler {
    pub fn new(spawner: JobSpawner<CustomerActiveSyncConfig>) -> Self {
        Self { spawner }
    }
}

impl<E> OutboxEventHandler<E> for CustomerActiveSyncHandler
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    #[instrument(name = "customer_sync.active_sync_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreCustomerEvent::CustomerCreated { entity }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.spawner
                .spawn_in_op(
                    op,
                    JobId::new(),
                    CustomerActiveSyncConfig {
                        customer_id: entity.id,
                    },
                )
                .await?;
        }
        Ok(())
    }
}
