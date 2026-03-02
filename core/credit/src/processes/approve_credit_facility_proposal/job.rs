use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_credit_collateral::{
    CoreCreditCollateralAction, CoreCreditCollateralObject, public::CoreCreditCollateralEvent,
};
use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_price::CorePriceEvent;
use governance::{ApprovalProcessId, GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    CoreCreditAction, CoreCreditCollectionAction, CoreCreditCollectionEvent,
    CoreCreditCollectionObject, CoreCreditEvent, CoreCreditObject,
};

use super::ApproveCreditFacilityProposal;

pub const APPROVE_CREDIT_FACILITY_PROPOSAL_COMMAND: JobType =
    JobType::new("command.credit.approve-credit-facility-proposal");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApproveCreditFacilityProposalConfig {
    pub approval_process_id: ApprovalProcessId,
    pub approved: bool,
}

pub struct ApproveCreditFacilityProposalJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    process: ApproveCreditFacilityProposal<Perms, E>,
}

impl<Perms, E> ApproveCreditFacilityProposalJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(process: &ApproveCreditFacilityProposal<Perms, E>) -> Self {
        Self {
            process: process.clone(),
        }
    }
}

impl<Perms, E> JobInitializer for ApproveCreditFacilityProposalJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<CoreCreditCollateralAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
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
    type Config = ApproveCreditFacilityProposalConfig;

    fn job_type(&self) -> JobType {
        APPROVE_CREDIT_FACILITY_PROPOSAL_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ApproveCreditFacilityProposalJobRunner {
            config: job.config()?,
            process: self.process.clone(),
        }))
    }
}

pub struct ApproveCreditFacilityProposalJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    config: ApproveCreditFacilityProposalConfig,
    process: ApproveCreditFacilityProposal<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for ApproveCreditFacilityProposalJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<CoreCreditCollateralAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
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
        name = "credit.approve_credit_facility_proposal.process_command",
        skip_all
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;
        self.process
            .execute_approve_credit_facility_proposal_in_op(
                &mut op,
                self.config.approval_process_id.into(),
                self.config.approved,
            )
            .await?;
        Ok(JobCompletion::CompleteWithOp(op))
    }
}
