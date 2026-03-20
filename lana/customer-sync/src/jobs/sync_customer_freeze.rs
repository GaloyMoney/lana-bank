use tracing::{Span, instrument};

use command_job::CommandJobSpawner;
use job::JobType;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use core_customer::CoreCustomerEvent;

use super::freeze_customer_deposits::FreezeCustomerDepositsCommand;

pub const CUSTOMER_FREEZE_SYNC: JobType = JobType::new("outbox.customer-freeze-sync");

pub struct SyncCustomerFreezeHandler {
    freeze_customer_deposits: CommandJobSpawner<FreezeCustomerDepositsCommand>,
}

impl SyncCustomerFreezeHandler {
    pub fn new(freeze_customer_deposits: CommandJobSpawner<FreezeCustomerDepositsCommand>) -> Self {
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
                .spawn_in_op(
                    op,
                    FreezeCustomerDepositsCommand {
                        customer_id: entity.id,
                        party_id: entity.party_id,
                    },
                )
                .await?;
        }
        Ok(())
    }
}
