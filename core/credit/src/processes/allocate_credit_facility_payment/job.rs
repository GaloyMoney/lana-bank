use tracing::{Span, instrument};

use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use crate::CoreCreditCollectionEvent;

use super::execute_allocate_payment::ExecuteAllocatePaymentConfig;

pub const ALLOCATE_CREDIT_FACILITY_PAYMENT: JobType =
    JobType::new("outbox.allocate-credit-facility-payment");

pub struct AllocateCreditFacilityPaymentHandler {
    execute_allocate_payment: JobSpawner<ExecuteAllocatePaymentConfig>,
}

impl AllocateCreditFacilityPaymentHandler {
    pub fn new(execute_allocate_payment: JobSpawner<ExecuteAllocatePaymentConfig>) -> Self {
        Self {
            execute_allocate_payment,
        }
    }
}

impl<E> OutboxEventHandler<E> for AllocateCreditFacilityPaymentHandler
where
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    #[instrument(name = "core_credit.allocate_credit_facility_payment_job.process_message_in_op", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty, credit_facility_id = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use CoreCreditCollectionEvent::*;

        if let Some(e @ PaymentCreated { entity }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());
            Span::current().record(
                "credit_facility_id",
                tracing::field::display(&entity.beneficiary_id),
            );

            self.execute_allocate_payment
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    ExecuteAllocatePaymentConfig {
                        payment_id: entity.id,
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        Ok(())
    }
}
