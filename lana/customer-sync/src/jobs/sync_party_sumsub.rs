use tracing::{Span, instrument};

use job::{JobId, JobSpawner, JobType};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use core_customer::CoreCustomerEvent;

use super::ingest_sumsub_applicant::IngestSumsubApplicantConfig;

pub const CUSTOMER_SYNC_INGEST_SUMSUB_APPLICANT: JobType =
    JobType::new("outbox.customer-sync-ingest-sumsub-applicant");

pub struct SyncPartySumsubHandler {
    ingest_sumsub_applicant: JobSpawner<IngestSumsubApplicantConfig>,
}

impl SyncPartySumsubHandler {
    pub fn new(ingest_sumsub_applicant: JobSpawner<IngestSumsubApplicantConfig>) -> Self {
        Self {
            ingest_sumsub_applicant,
        }
    }
}

impl<E> OutboxEventHandler<E> for SyncPartySumsubHandler
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    #[instrument(name = "customer_sync.ingest_sumsub_applicant_sync_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match event.as_event() {
            Some(e @ CoreCustomerEvent::PartyCreated { entity })
            | Some(e @ CoreCustomerEvent::PartyPersonalInfoUpdated { entity }) => {
                event.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", e.as_ref());

                self.ingest_sumsub_applicant
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        IngestSumsubApplicantConfig { party_id: entity.id },
                        entity.id.to_string(),
                    )
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}
