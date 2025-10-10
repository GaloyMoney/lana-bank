mod job;

use std::sync::Arc;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{
    ApprovalProcess, ApprovalProcessStatus, ApprovalProcessType, Governance, GovernanceAction,
    GovernanceEvent, GovernanceObject,
};
use outbox::OutboxEventMarker;

use crate::{
    CoreCreditAction, CoreCreditEvent, CoreCreditObject, CreditFacilityProposal,
    CreditFacilityProposalId, CreditFacilityProposals, error::CoreCreditError,
};

pub use job::*;
pub const APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS: ApprovalProcessType =
    ApprovalProcessType::new("credit-facility-proposal");

pub struct ApproveCreditFacilityProposal<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    credit_facility_proposals: Arc<CreditFacilityProposals<Perms, E>>,
    audit: Arc<Perms::Audit>,
    governance: Arc<Governance<Perms, E>>,
}

impl<Perms, E> Clone for ApproveCreditFacilityProposal<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            credit_facility_proposals: self.credit_facility_proposals.clone(),
            audit: self.audit.clone(),
            governance: self.governance.clone(),
        }
    }
}

impl<Perms, E> ApproveCreditFacilityProposal<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(
        repo: Arc<CreditFacilityProposals<Perms, E>>,
        audit: Arc<Perms::Audit>,
        governance: Arc<Governance<Perms, E>>,
    ) -> Self {
        Self {
            credit_facility_proposals: repo,
            audit,
            governance,
        }
    }

    pub async fn execute_from_svc(
        &self,
        credit_facility_proposal: &CreditFacilityProposal,
    ) -> Result<Option<CreditFacilityProposal>, CoreCreditError> {
        if credit_facility_proposal.is_approval_process_concluded() {
            return Ok(None);
        }

        let process: ApprovalProcess = self
            .governance
            .find_all_approval_processes(&[credit_facility_proposal.approval_process_id])
            .await?
            .remove(&credit_facility_proposal.approval_process_id)
            .expect("approval process not found");

        let res = match process.status() {
            ApprovalProcessStatus::Approved => {
                Some(self.execute(credit_facility_proposal.id, true).await?)
            }
            ApprovalProcessStatus::Denied => {
                Some(self.execute(credit_facility_proposal.id, false).await?)
            }
            _ => None,
        };
        Ok(res)
    }

    #[instrument(name = "credit_facility.approval.execute", skip(self))]
    pub async fn execute(
        &self,
        id: impl es_entity::RetryableInto<CreditFacilityProposalId>,
        approved: bool,
    ) -> Result<CreditFacilityProposal, CoreCreditError> {
        let credit_facility = self
            .credit_facility_proposals
            .approve(id.into(), approved)
            .await?;
        Ok(credit_facility)
    }
}
