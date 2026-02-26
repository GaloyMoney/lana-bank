use tracing::{Span, instrument};

use governance::GovernanceEvent;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use super::APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS;
use super::execute_approve_credit_facility_proposal::ExecuteApproveCreditFacilityProposalConfig;

pub const CREDIT_FACILITY_PROPOSAL_APPROVE_JOB: JobType =
    JobType::new("outbox.credit-facility-proposal-approval");

pub(crate) struct CreditFacilityProposalApprovalHandler {
    execute_approve_credit_facility_proposal:
        JobSpawner<ExecuteApproveCreditFacilityProposalConfig>,
}

impl CreditFacilityProposalApprovalHandler {
    pub fn new(
        execute_approve_credit_facility_proposal: JobSpawner<
            ExecuteApproveCreditFacilityProposalConfig,
        >,
    ) -> Self {
        Self {
            execute_approve_credit_facility_proposal,
        }
    }
}

impl<E> OutboxEventHandler<E> for CreditFacilityProposalApprovalHandler
where
    E: OutboxEventMarker<GovernanceEvent>,
{
    #[instrument(name = "core_credit.credit_facility_proposal_approval_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty, process_type = tracing::field::Empty, credit_facility_proposal_id = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match event.as_event() {
            Some(e @ GovernanceEvent::ApprovalProcessConcluded { entity })
                if entity.process_type == APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS =>
            {
                event.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", e.as_ref());
                Span::current().record(
                    "credit_facility_proposal_id",
                    tracing::field::display(entity.id),
                );
                Span::current().record("process_type", entity.process_type.to_string());
                self.execute_approve_credit_facility_proposal
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        ExecuteApproveCreditFacilityProposalConfig {
                            approval_process_id: entity.id,
                            approved: entity.status.is_approved(),
                            trace_context: tracing_utils::persistence::extract(),
                        },
                        entity.id.to_string(),
                    )
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}
