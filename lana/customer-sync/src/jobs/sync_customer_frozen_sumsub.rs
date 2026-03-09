use tracing::{Span, instrument};

use job::{JobId, JobSpawner, JobType};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use core_customer::CoreCustomerEvent;

use super::reject_sumsub_applicant::RejectSumsubApplicantConfig;

pub const CUSTOMER_SYNC_REJECT_SUMSUB_APPLICANT: JobType =
    JobType::new("outbox.customer-sync-reject-sumsub-applicant");

pub struct SyncCustomerFrozenSumsubHandler {
    reject_sumsub_applicant: JobSpawner<RejectSumsubApplicantConfig>,
}

impl SyncCustomerFrozenSumsubHandler {
    pub fn new(reject_sumsub_applicant: JobSpawner<RejectSumsubApplicantConfig>) -> Self {
        Self {
            reject_sumsub_applicant,
        }
    }
}

impl<E> OutboxEventHandler<E> for SyncCustomerFrozenSumsubHandler
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    #[instrument(name = "customer_sync.reject_sumsub_applicant_sync_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreCustomerEvent::CustomerFrozen { entity }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.reject_sumsub_applicant
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    RejectSumsubApplicantConfig {
                        customer_id: entity.id,
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        Ok(())
    }
}
