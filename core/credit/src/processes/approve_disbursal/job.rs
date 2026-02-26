use tracing::{Span, instrument};

use governance::GovernanceEvent;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use super::APPROVE_DISBURSAL_PROCESS;
use super::execute_approve_disbursal::ExecuteApproveDisbursalConfig;

pub const DISBURSAL_APPROVE_JOB: JobType = JobType::new("outbox.disbursal-approval");

pub(crate) struct DisbursalApprovalHandler {
    execute_approve_disbursal: JobSpawner<ExecuteApproveDisbursalConfig>,
}

impl DisbursalApprovalHandler {
    pub fn new(execute_approve_disbursal: JobSpawner<ExecuteApproveDisbursalConfig>) -> Self {
        Self {
            execute_approve_disbursal,
        }
    }
}

impl<E> OutboxEventHandler<E> for DisbursalApprovalHandler
where
    E: OutboxEventMarker<GovernanceEvent>,
{
    #[instrument(name = "core_credit.disbursal_approval_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty, process_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match event.as_event() {
            Some(e @ GovernanceEvent::ApprovalProcessConcluded { entity })
                if entity.process_type == APPROVE_DISBURSAL_PROCESS =>
            {
                event.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", e.as_ref());
                Span::current().record("process_type", entity.process_type.to_string());
                self.execute_approve_disbursal
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        ExecuteApproveDisbursalConfig {
                            approval_process_id: entity.id,
                            approved: entity.status.is_approved(),
                            trace_context: Some(tracing_utils::persistence::extract()),
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
