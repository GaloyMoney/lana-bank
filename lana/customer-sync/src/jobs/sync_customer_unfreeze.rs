use tracing::{Span, instrument};

use job::{JobId, JobSpawner, JobType};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use core_customer::CoreCustomerEvent;

use super::unfreeze_customer_deposits::UnfreezeCustomerDepositsConfig;

pub const CUSTOMER_UNFREEZE_SYNC: JobType = JobType::new("outbox.customer-unfreeze-sync");

pub struct SyncCustomerUnfreezeHandler {
    unfreeze_customer_deposits: JobSpawner<UnfreezeCustomerDepositsConfig>,
}

impl SyncCustomerUnfreezeHandler {
    pub fn new(unfreeze_customer_deposits: JobSpawner<UnfreezeCustomerDepositsConfig>) -> Self {
        Self {
            unfreeze_customer_deposits,
        }
    }
}

impl<E> OutboxEventHandler<E> for SyncCustomerUnfreezeHandler
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    #[instrument(name = "customer_sync.unfreeze_sync_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreCustomerEvent::CustomerUnfrozen { entity }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.unfreeze_customer_deposits
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    UnfreezeCustomerDepositsConfig {
                        customer_id: entity.id,
                        party_id: entity.party_id,
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        Ok(())
    }
}
