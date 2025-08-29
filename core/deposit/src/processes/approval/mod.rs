mod job;

use authz::PermissionCheck;
use governance::{
    ApprovalProcess, ApprovalProcessStatus, ApprovalProcessType, GovernanceAction, GovernanceEvent,
    GovernanceObject,
};

use audit::AuditSvc;
use governance::Governance;
use outbox::OutboxEventMarker;

use crate::{
    CoreDepositAction, CoreDepositObject, WithdrawalAction,
    event::CoreDepositEvent,
    primitives::WithdrawalId,
    withdrawal::{Withdrawal, error::WithdrawalError, repo::WithdrawalRepo},
};

use super::error::ProcessError;

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

    pub async fn execute_from_svc(
        &self,
        withdraw: &Withdrawal,
    ) -> Result<Option<Withdrawal>, ProcessError> {
        if withdraw.is_approved_or_denied().is_some() {
            return Ok(None);
        }

        let process: ApprovalProcess = self
            .governance
            .find_all_approval_processes(&[withdraw.approval_process_id])
            .await?
            .remove(&withdraw.approval_process_id)
            .expect("approval process not found");

        let res = match process.status() {
            ApprovalProcessStatus::Approved => Some(self.execute(withdraw.id, true).await?),
            ApprovalProcessStatus::Denied => Some(self.execute(withdraw.id, false).await?),
            _ => None,
        };
        Ok(res)
    }

    #[es_entity::retry_on_concurrent_modification]
    pub async fn execute(
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
