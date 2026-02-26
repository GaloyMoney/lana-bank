use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_price::CorePriceEvent;
use governance::{ApprovalProcessId, GovernanceAction, GovernanceEvent, GovernanceObject};
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    CoreCreditAction, CoreCreditEvent, CoreCreditObject,
    collateral::{
        CoreCreditCollateralAction, CoreCreditCollateralObject, public::CoreCreditCollateralEvent,
    },
};
use core_credit_collection::CoreCreditCollectionEvent;

use super::ApproveCreditFacilityProposal;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteApproveCreditFacilityProposalConfig {
    pub approval_process_id: ApprovalProcessId,
    pub approved: bool,
    #[serde(default)]
    pub trace_context: Option<tracing_utils::persistence::SerializableTraceContext>,
}

pub const EXECUTE_APPROVE_CREDIT_FACILITY_PROPOSAL_COMMAND: JobType =
    JobType::new("command.credit.execute-approve-credit-facility-proposal");

pub struct ExecuteApproveCreditFacilityProposalJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    process: ApproveCreditFacilityProposal<Perms, E>,
}

impl<Perms, E> ExecuteApproveCreditFacilityProposalJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(process: ApproveCreditFacilityProposal<Perms, E>) -> Self {
        Self { process }
    }
}

impl<Perms, E> JobInitializer for ExecuteApproveCreditFacilityProposalJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<crate::CoreCreditCollectionAction>
        + From<CoreCreditCollateralAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<crate::CoreCreditCollectionObject>
        + From<CoreCreditCollateralObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    type Config = ExecuteApproveCreditFacilityProposalConfig;

    fn job_type(&self) -> JobType {
        EXECUTE_APPROVE_CREDIT_FACILITY_PROPOSAL_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ExecuteApproveCreditFacilityProposalJobRunner {
            config: job.config()?,
            process: self.process.clone(),
        }))
    }
}

pub struct ExecuteApproveCreditFacilityProposalJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    config: ExecuteApproveCreditFacilityProposalConfig,
    process: ApproveCreditFacilityProposal<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for ExecuteApproveCreditFacilityProposalJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<crate::CoreCreditCollectionAction>
        + From<CoreCreditCollateralAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<crate::CoreCreditCollectionObject>
        + From<CoreCreditCollateralObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(
        name = "core_credit.execute_approve_credit_facility_proposal_job.process_command",
        skip(self, _current_job),
        fields(approval_process_id = %self.config.approval_process_id, approved = %self.config.approved),
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        if let Some(ref ctx) = self.config.trace_context {
            tracing_utils::persistence::set_parent(ctx);
        }
        self.process
            .execute_approve_credit_facility_proposal(
                self.config.approval_process_id,
                self.config.approved,
            )
            .await?;

        Ok(JobCompletion::Complete)
    }
}
