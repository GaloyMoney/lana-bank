use tracing::{Span, instrument};

use job::{JobId, JobSpawner, JobType};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use core_customer::CoreCustomerEvent;

use super::command_job::UpdateAccountStatusForHolderConfig;

pub const CUSTOMER_ACTIVE_SYNC: JobType = JobType::new("outbox.customer-active-sync");

pub struct CustomerActiveSyncHandler {
    update_account_status_for_holder_job_spawner: JobSpawner<UpdateAccountStatusForHolderConfig>,
}

impl CustomerActiveSyncHandler {
    pub fn new(
        update_account_status_for_holder_job_spawner: JobSpawner<
            UpdateAccountStatusForHolderConfig,
        >,
    ) -> Self {
        Self {
            update_account_status_for_holder_job_spawner,
        }
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

            self.update_account_status_for_holder_job_spawner
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    UpdateAccountStatusForHolderConfig {
                        customer_id: entity.id,
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        Ok(())
    }
}
