use tracing::{Span, instrument};

use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use crate::{CoreCreditEvent, PendingCreditFacilityCollateralizationState};

use super::execute_activate_credit_facility::ExecuteActivateCreditFacilityConfig;

pub const CREDIT_FACILITY_ACTIVATE: JobType = JobType::new("outbox.credit-facility-activation");

pub(crate) struct CreditFacilityActivationHandler {
    execute_activate_credit_facility: JobSpawner<ExecuteActivateCreditFacilityConfig>,
}

impl CreditFacilityActivationHandler {
    pub fn new(
        execute_activate_credit_facility: JobSpawner<ExecuteActivateCreditFacilityConfig>,
    ) -> Self {
        Self {
            execute_activate_credit_facility,
        }
    }
}

impl<E> OutboxEventHandler<E> for CreditFacilityActivationHandler
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[instrument(name = "core_credit.credit_facility_activation_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty, pending_credit_facility_id = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use CoreCreditEvent::*;

        if let Some(e @ PendingCreditFacilityCollateralizationChanged { entity }) = event.as_event()
            && entity.collateralization.state
                == PendingCreditFacilityCollateralizationState::FullyCollateralized
        {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record(
                "pending_credit_facility_id",
                tracing::field::display(entity.id),
            );
            Span::current().record("event_type", e.as_ref());

            self.execute_activate_credit_facility
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    ExecuteActivateCreditFacilityConfig {
                        pending_credit_facility_id: entity.id,
                        trace_context: Some(tracing_utils::persistence::extract()),
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        Ok(())
    }
}
