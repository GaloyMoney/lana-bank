use tracing::{Span, instrument};

use governance::GovernanceEvent;
use job::{JobId, JobSpawner, JobType};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use super::ExecuteApproveDisbursalConfig;

pub const DISBURSAL_APPROVE_JOB: JobType = JobType::new("outbox.disbursal-approval");

pub struct DisbursalApprovalHandler {
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
        if let Some(e @ GovernanceEvent::ApprovalProcessConcluded { entity }) = event.as_event()
            && entity.process_type == super::APPROVE_DISBURSAL_PROCESS
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
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        Ok(())
    }
}
