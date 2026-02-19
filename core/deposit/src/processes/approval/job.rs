use authz::PermissionCheck;
use tracing::{Span, instrument};

use audit::AuditSvc;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::JobType;

use crate::{CoreDepositAction, CoreDepositObject, public::CoreDepositEvent};

use super::ApproveWithdrawal;

pub const WITHDRAW_APPROVE_JOB: JobType = JobType::new("outbox.withdraw-approval");

pub struct WithdrawApprovalHandler<Perms, E>
where
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreDepositEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<GovernanceObject>,
{
    process: ApproveWithdrawal<Perms, E>,
}

impl<Perms, E> WithdrawApprovalHandler<Perms, E>
where
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreDepositEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<GovernanceObject>,
{
    pub fn new(process: &ApproveWithdrawal<Perms, E>) -> Self {
        Self {
            process: process.clone(),
        }
    }
}

impl<Perms, E> OutboxEventHandler<E> for WithdrawApprovalHandler<Perms, E>
where
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreDepositEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<GovernanceObject>,
{
    #[instrument(name = "core_deposit.withdraw_approval_job.process_message", parent = None, skip(self, _op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty, process_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        _op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ GovernanceEvent::ApprovalProcessConcluded { entity }) = event.as_event()
            && entity.process_type == super::APPROVE_WITHDRAWAL_PROCESS
        {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());
            Span::current().record("process_type", entity.process_type.to_string());
            self.process
                .execute_withdrawal_approval(entity.id, entity.status.is_approved())
                .await?;
        }
        Ok(())
    }
}
