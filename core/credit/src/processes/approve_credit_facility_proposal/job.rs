use async_trait::async_trait;
use futures::StreamExt;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use outbox::{Outbox, OutboxEventMarker};

use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};

use crate::{CoreCreditAction, CoreCreditEvent, CoreCreditObject};

use super::ApproveCreditFacilityProposal;

#[derive(serde::Serialize)]
pub(crate) struct CreditFacilityProposalApprovalJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> CreditFacilityProposalApprovalJobConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> Default for CreditFacilityProposalApprovalJobConfig<Perms, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Perms, E> JobConfig for CreditFacilityProposalApprovalJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    type Initializer = CreditFacilityProposalApprovalInit<Perms, E>;
}

pub(crate) struct CreditFacilityProposalApprovalInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
    process: ApproveCreditFacilityProposal<Perms, E>,
}

impl<Perms, E> CreditFacilityProposalApprovalInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(outbox: &Outbox<E>, process: &ApproveCreditFacilityProposal<Perms, E>) -> Self {
        Self {
            process: process.clone(),
            outbox: outbox.clone(),
        }
    }
}

const CREDIT_FACILITY_PROPOSAL_APPROVE_JOB: JobType =
    JobType::new("credit-facility-proposal-approval");
impl<Perms, E> JobInitializer for CreditFacilityProposalApprovalInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_PROPOSAL_APPROVE_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityProposalApprovalJobRunner {
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
struct CreditFacilityProposalApprovalJobData {
    sequence: outbox::EventSequence,
}

pub struct CreditFacilityProposalApprovalJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
    process: ApproveCreditFacilityProposal<Perms, E>,
}
#[async_trait]
impl<Perms, E> JobRunner for CreditFacilityProposalApprovalJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CreditFacilityProposalApprovalJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            match message.as_ref().as_event() {
                Some(GovernanceEvent::ApprovalProcessConcluded {
                    id,
                    approved,
                    process_type,
                    ..
                }) if process_type == &super::APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS => {
                    self.process.execute(*id, *approved).await?;
                    state.sequence = message.sequence;
                    current_job.update_execution_state(state).await?;
                }
                _ => {}
            }
        }

        Ok(JobCompletion::RescheduleNow)
    }
}
