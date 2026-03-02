use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{Span, instrument};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_credit_collateral::{
    CoreCreditCollateralAction, CoreCreditCollateralObject, public::CoreCreditCollateralEvent,
};
use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_price::CorePriceEvent;
use governance::{ApprovalProcessId, GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};
use tracing_macros::record_error_severity;

use crate::{
    CoreCreditAction, CoreCreditCollectionAction, CoreCreditCollectionEvent,
    CoreCreditCollectionObject, CoreCreditEvent, CoreCreditObject,
};

use super::ApproveCreditFacilityProposal;

pub const CREDIT_FACILITY_PROPOSAL_APPROVE_JOB: JobType =
    JobType::new("outbox.credit-facility-proposal-approval");

pub struct CreditFacilityProposalApprovalHandler {
    approve_credit_facility_proposal: JobSpawner<ApproveCreditFacilityProposalConfig>,
}

impl CreditFacilityProposalApprovalHandler {
    pub fn new(
        approve_credit_facility_proposal: JobSpawner<ApproveCreditFacilityProposalConfig>,
    ) -> Self {
        Self {
            approve_credit_facility_proposal,
        }
    }
}

impl<E> OutboxEventHandler<E> for CreditFacilityProposalApprovalHandler
where
    E: OutboxEventMarker<GovernanceEvent>,
{
    #[instrument(name = "core_credit.credit_facility_proposal_approval_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty, process_type = tracing::field::Empty, credit_facility_proposal_id = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ GovernanceEvent::ApprovalProcessConcluded { entity }) = event.as_event()
            && entity.process_type == super::APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS
        {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());
            Span::current().record(
                "credit_facility_proposal_id",
                tracing::field::display(entity.id),
            );
            Span::current().record("process_type", entity.process_type.to_string());
            self.approve_credit_facility_proposal
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    ApproveCreditFacilityProposalConfig {
                        approval_process_id: entity.id,
                        approved: entity.status.is_approved(),
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        Ok(())
    }
}

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
