use tracing::{Span, instrument};

use governance::GovernanceEvent;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use super::APPROVE_WITHDRAWAL_PROCESS;
use super::execute_withdraw_approval::ExecuteWithdrawApprovalConfig;

pub const WITHDRAW_APPROVE_JOB: JobType = JobType::new("outbox.withdraw-approval");

pub struct WithdrawApprovalHandler {
    execute_withdraw_approval: JobSpawner<ExecuteWithdrawApprovalConfig>,
}

impl WithdrawApprovalHandler {
    pub fn new(execute_withdraw_approval: JobSpawner<ExecuteWithdrawApprovalConfig>) -> Self {
        Self {
            execute_withdraw_approval,
        }
    }
}

impl<E> OutboxEventHandler<E> for WithdrawApprovalHandler
where
    E: OutboxEventMarker<GovernanceEvent>,
{
    #[instrument(name = "core_deposit.withdraw_approval_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty, process_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ GovernanceEvent::ApprovalProcessConcluded { entity }) = event.as_event()
            && entity.process_type == APPROVE_WITHDRAWAL_PROCESS
        {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());
            Span::current().record("process_type", entity.process_type.to_string());
            self.execute_withdraw_approval
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    ExecuteWithdrawApprovalConfig {
                        approval_process_id: entity.id,
                        approved: entity.status.is_approved(),
                        trace_context: tracing_utils::persistence::extract(),
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        Ok(())
    }
}
