mod job;

use authz::PermissionCheck;
use governance::{ApprovalProcessType, GovernanceAction, GovernanceEvent, GovernanceObject};
use tracing::instrument;

use audit::AuditSvc;
use governance::Governance;
use outbox::OutboxEventMarker;

use crate::{
    CoreDepositAction, CoreDepositObject, WithdrawalAction,
    event::CoreDepositEvent,
    primitives::WithdrawalId,
    withdrawal::{Withdrawal, error::WithdrawalError, repo::WithdrawalRepo},
};

pub use job::*;

pub const APPROVE_WITHDRAWAL_PROCESS: ApprovalProcessType = ApprovalProcessType::new("withdraw");

pub struct ApproveWithdrawal<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    repo: WithdrawalRepo<E>,
    audit: Perms::Audit,
    governance: Governance<Perms, E>,
}
impl<Perms, E> Clone for ApproveWithdrawal<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            audit: self.audit.clone(),
            governance: self.governance.clone(),
        }
    }
}

impl<Perms, E> ApproveWithdrawal<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    pub fn new(
        repo: &WithdrawalRepo<E>,
        audit: &Perms::Audit,
        governance: &Governance<Perms, E>,
    ) -> Self {
        Self {
            repo: repo.clone(),
            audit: audit.clone(),
            governance: governance.clone(),
        }
    }

    #[instrument(name = "core_deposit.withdrawal_approval.execute", skip(self))]
    #[es_entity::retry_on_concurrent_modification]
    pub async fn execute_withdrawal_approval(
        &self,
        id: impl es_entity::RetryableInto<WithdrawalId>,
        approved: bool,
    ) -> Result<Withdrawal, WithdrawalError> {
        let id = id.into();
        let mut withdraw = self.repo.find_by_id(id).await?;
        if withdraw.is_approved_or_denied().is_some() {
            return Ok(withdraw);
        }
        let mut op = self.repo.begin_op().await?;
        self.audit
            .record_system_entry_in_tx(
                &mut op,
                CoreDepositObject::withdrawal(id),
                CoreDepositAction::Withdrawal(WithdrawalAction::ConcludeApprovalProcess),
            )
            .await?;
        if withdraw.approval_process_concluded(approved).did_execute() {
            self.repo.update_in_op(&mut op, &mut withdraw).await?;
            op.commit().await?;
        }
        Ok(withdraw)
    }
}
