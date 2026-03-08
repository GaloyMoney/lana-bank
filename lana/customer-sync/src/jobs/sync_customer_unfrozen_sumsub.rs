use tracing::{Span, instrument};

use job::{JobId, JobSpawner, JobType};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use core_customer::CoreCustomerEvent;

use super::approve_sumsub_applicant::ApproveSumsubApplicantConfig;

pub const CUSTOMER_SYNC_APPROVE_SUMSUB_APPLICANT: JobType =
    JobType::new("outbox.customer-sync-approve-sumsub-applicant");

pub struct SyncCustomerUnfrozenSumsubHandler {
    approve_sumsub_applicant: JobSpawner<ApproveSumsubApplicantConfig>,
}

impl SyncCustomerUnfrozenSumsubHandler {
    pub fn new(approve_sumsub_applicant: JobSpawner<ApproveSumsubApplicantConfig>) -> Self {
        Self {
            approve_sumsub_applicant,
        }
    }
}

impl<E> OutboxEventHandler<E> for SyncCustomerUnfrozenSumsubHandler
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    #[instrument(name = "customer_sync.approve_sumsub_applicant_sync_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreCustomerEvent::CustomerUnfrozen { entity }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.approve_sumsub_applicant
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    ApproveSumsubApplicantConfig {
                        customer_id: entity.id,
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        Ok(())
    }
}
