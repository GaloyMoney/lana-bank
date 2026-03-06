use tracing::{Span, instrument};

use job::{JobId, JobSpawner, JobType};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use core_customer::CoreCustomerEvent;

use super::freeze_customer_deposits::FreezeCustomerDepositsConfig;

pub const CUSTOMER_FREEZE_SYNC: JobType = JobType::new("outbox.customer-freeze-sync");

pub struct SyncCustomerFreezeHandler {
    freeze_customer_deposits: JobSpawner<FreezeCustomerDepositsConfig>,
}

impl SyncCustomerFreezeHandler {
    pub fn new(freeze_customer_deposits: JobSpawner<FreezeCustomerDepositsConfig>) -> Self {
        Self {
            freeze_customer_deposits,
        }
    }
}

impl<E> OutboxEventHandler<E> for SyncCustomerFreezeHandler
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    #[instrument(name = "customer_sync.freeze_sync_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreCustomerEvent::CustomerFrozen { entity }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.freeze_customer_deposits
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    FreezeCustomerDepositsConfig {
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
