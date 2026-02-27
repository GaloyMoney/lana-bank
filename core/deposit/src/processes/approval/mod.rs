mod execute_withdraw_approval;
mod withdraw_approval;

use authz::PermissionCheck;
use governance::{ApprovalProcessType, GovernanceAction, GovernanceEvent, GovernanceObject};
use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::{AuditSvc, SystemSubject};
use governance::Governance;
use obix::out::OutboxEventMarker;

use crate::{
    CoreDepositAction, CoreDepositObject, WithdrawalAction,
    ledger::DepositLedger,
    primitives::WithdrawalId,
    public::CoreDepositEvent,
    withdrawal::{Withdrawal, error::WithdrawalError, repo::WithdrawalRepo},
};

pub use execute_withdraw_approval::*;
pub use withdraw_approval::*;

pub const APPROVE_WITHDRAWAL_PROCESS: ApprovalProcessType = ApprovalProcessType::new("withdraw");

pub struct ApproveWithdrawal<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    repo: WithdrawalRepo<E>,
    audit: Perms::Audit,
    governance: Governance<Perms, E>,
    ledger: DepositLedger,
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
            ledger: self.ledger.clone(),
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
        ledger: &DepositLedger,
    ) -> Self {
        Self {
            repo: repo.clone(),
            audit: audit.clone(),
            governance: governance.clone(),
            ledger: ledger.clone(),
        }
    }

    #[record_error_severity]
    #[instrument(name = "core_deposit.withdrawal_approval.execute", skip(self, op))]
    pub async fn execute_withdrawal_approval_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        id: WithdrawalId,
        approved: bool,
    ) -> Result<Withdrawal, WithdrawalError> {
        let id = id.into();
        let mut withdraw = self.repo.find_by_id_in_op(op, id).await?;
        if withdraw.is_approved_or_denied().is_some() {
            return Ok(withdraw);
        }
        self.audit
            .record_system_entry_in_op(
                op,
                crate::primitives::DEPOSIT_APPROVAL,
                CoreDepositObject::withdrawal(id),
                CoreDepositAction::Withdrawal(WithdrawalAction::ConcludeApprovalProcess),
            )
            .await?;
        match withdraw.approval_process_concluded(approved) {
            es_entity::Idempotent::Executed(Some(denied_tx_id)) => {
                self.repo.update_in_op(op, &mut withdraw).await?;
                self.ledger
                    .deny_withdrawal_in_op(
                        op,
                        withdraw.id,
                        denied_tx_id,
                        withdraw.amount,
                        withdraw.deposit_account_id,
                        &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(
                            crate::primitives::DEPOSIT_APPROVAL,
                        ),
                    )
                    .await?;
            }
            es_entity::Idempotent::Executed(None) => {
                self.repo.update_in_op(op, &mut withdraw).await?;
            }
            _ => (),
        };
        Ok(withdraw)
    }
}
