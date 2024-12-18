use async_trait::async_trait;
use authz::PermissionCheck;
use futures::StreamExt;

use audit::AuditSvc;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use outbox::{Outbox, OutboxEventMarker};

use crate::{CoreDepositAction, CoreDepositObject};

use super::ApproveWithdrawal;

#[derive(serde::Serialize)]
pub struct WithdrawApprovalJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> WithdrawApprovalJobConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<Perms, E> JobConfig for WithdrawApprovalJobConfig<Perms, E>
where
    E: OutboxEventMarker<GovernanceEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<GovernanceObject>,
{
    type Initializer = WithdrawApprovalJobInitializer<Perms, E>;
}

pub struct WithdrawApprovalJobInitializer<Perms, E>
where
    E: OutboxEventMarker<GovernanceEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<GovernanceObject>,
{
    outbox: Outbox<E>,
    process: ApproveWithdrawal<Perms, E>,
}

impl<Perms, E> WithdrawApprovalJobInitializer<Perms, E>
where
    E: OutboxEventMarker<GovernanceEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<GovernanceObject>,
{
    pub fn new(outbox: &Outbox<E>, process: &ApproveWithdrawal<Perms, E>) -> Self {
        Self {
            process: process.clone(),
            outbox: outbox.clone(),
        }
    }
}

const WITHDRAW_APPROVE_JOB: JobType = JobType::new("withdraw-approval");
impl<Perms, E> JobInitializer for WithdrawApprovalJobInitializer<Perms, E>
where
    E: OutboxEventMarker<GovernanceEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<GovernanceObject>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        WITHDRAW_APPROVE_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(WithdrawApprovalJobRunner {
            outbox: self.outbox.clone(),
            process: self.process.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct WithdrawApprovalJobData {
    sequence: outbox::EventSequence,
}

pub struct WithdrawApprovalJobRunner<Perms, E>
where
    E: OutboxEventMarker<GovernanceEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<GovernanceObject>,
{
    outbox: Outbox<E>,
    process: ApproveWithdrawal<Perms, E>,
}
#[async_trait]
impl<Perms, E> JobRunner for WithdrawApprovalJobRunner<Perms, E>
where
    E: OutboxEventMarker<GovernanceEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<GovernanceObject>,
{
    #[allow(clippy::single_match)]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<WithdrawApprovalJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            match message.as_ref().as_event() {
                Some(GovernanceEvent::ApprovalProcessConcluded {
                    id,
                    approved,
                    ref process_type,
                    ..
                }) if process_type == &super::APPROVE_WITHDRAWAL_PROCESS => {
                    self.process.execute(*id, *approved).await?;
                    state.sequence = message.sequence;
                    current_job.update_execution_state(state).await?;
                }
                _ => {}
            }
        }

        Ok(JobCompletion::RescheduleAt(chrono::Utc::now()))
    }
}
