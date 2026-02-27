use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    CoreDepositAction, CoreDepositObject, primitives::ApprovalProcessId, public::CoreDepositEvent,
};

use super::ApproveWithdrawal;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteWithdrawApprovalConfig {
    pub approval_process_id: ApprovalProcessId,
    pub approved: bool,
}

pub const EXECUTE_WITHDRAW_APPROVAL_COMMAND: JobType =
    JobType::new("command.deposit.execute-withdraw-approval");

pub struct ExecuteWithdrawApprovalJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    process: ApproveWithdrawal<Perms, E>,
}

impl<Perms, E> ExecuteWithdrawApprovalJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    pub fn new(process: ApproveWithdrawal<Perms, E>) -> Self {
        Self { process }
    }
}

impl<Perms, E> JobInitializer for ExecuteWithdrawApprovalJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    type Config = ExecuteWithdrawApprovalConfig;

    fn job_type(&self) -> JobType {
        EXECUTE_WITHDRAW_APPROVAL_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ExecuteWithdrawApprovalJobRunner {
            config: job.config()?,
            process: self.process.clone(),
        }))
    }
}

pub struct ExecuteWithdrawApprovalJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    config: ExecuteWithdrawApprovalConfig,
    process: ApproveWithdrawal<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for ExecuteWithdrawApprovalJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(
        name = "core_deposit.execute_withdraw_approval_job.process_command",
        skip(self, _current_job),
        fields(approval_process_id = %self.config.approval_process_id, approved = %self.config.approved),
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        self.process
            .execute_withdrawal_approval(self.config.approval_process_id, self.config.approved)
            .await?;

        Ok(JobCompletion::Complete)
    }
}
