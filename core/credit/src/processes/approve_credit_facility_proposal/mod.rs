mod job;

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
    CreditFacilityProposalId, CreditFacilityProposals, PendingCreditFacilities,
    error::CoreCreditError,
};

use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};

pub use job::*;
pub const APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS: ApprovalProcessType =
    ApprovalProcessType::new("credit-facility-proposal");

pub struct ApproveCreditFacilityProposal<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    proposals: CreditFacilityProposals<Perms, E>,
    pending_credit_facilities: PendingCreditFacilities<Perms, E>,
    audit: Perms::Audit,
    governance: Governance<Perms, E>,
}

impl<Perms, E> Clone for ApproveCreditFacilityProposal<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    fn clone(&self) -> Self {
        Self {
            proposals: self.proposals.clone(),
            pending_credit_facilities: self.pending_credit_facilities.clone(),
            audit: self.audit.clone(),
            governance: self.governance.clone(),
        }
    }
}

impl<Perms, E> ApproveCreditFacilityProposal<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(
        proposals: &CreditFacilityProposals<Perms, E>,
        pending_credit_facilities: &PendingCreditFacilities<Perms, E>,
        audit: &Perms::Audit,
        governance: &Governance<Perms, E>,
    ) -> Self {
        Self {
            proposals: proposals.clone(),
            pending_credit_facilities: pending_credit_facilities.clone(),
            audit: audit.clone(),
            governance: governance.clone(),
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

    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    #[instrument(name = "credit_facility.approval.execute", skip(self))]
    pub async fn execute(
        &self,
        id: impl es_entity::RetryableInto<CreditFacilityProposalId>,
        approved: bool,
    ) -> Result<CreditFacilityProposal, CoreCreditError> {
        let mut db = self.proposals.begin_op().await?;
        let proposal = self.proposals.approve(&mut db, id.into(), approved).await?;

        if approved
            && matches!(
                self.pending_credit_facilities.find_by_id_without_audit(&proposal.id.into()).await,
                Err(ref e) if e.was_not_found()
            )
        {
            self.pending_credit_facilities
                .create_in_op(db, &proposal)
                .await?;
        } else {
            db.commit().await?;
        }

        Ok(proposal)
    }
}
