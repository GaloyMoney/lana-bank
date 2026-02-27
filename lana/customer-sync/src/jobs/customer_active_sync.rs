use tracing::{Span, instrument};

use command_job::CommandJobSpawner;
use job::JobType;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use core_customer::CoreCustomerEvent;

use super::activate_holder_account::ActivateHolderAccountCommand;

pub const CUSTOMER_ACTIVE_SYNC: JobType = JobType::new("outbox.customer-active-sync");

pub struct CustomerActiveSyncHandler {
    activate_holder_account: CommandJobSpawner<ActivateHolderAccountCommand>,
}

impl CustomerActiveSyncHandler {
    pub fn new(activate_holder_account: CommandJobSpawner<ActivateHolderAccountCommand>) -> Self {
        Self {
            activate_holder_account,
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

            self.activate_holder_account
                .spawn_in_op(
                    op,
                    ActivateHolderAccountCommand {
                        customer_id: entity.id,
                    },
                )
                .await?;
        }
        Ok(())
    }
}
