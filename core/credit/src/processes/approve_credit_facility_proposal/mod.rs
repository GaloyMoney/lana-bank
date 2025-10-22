mod job;

use std::sync::Arc;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{
    ApprovalProcessType, Governance, GovernanceAction, GovernanceEvent, GovernanceObject,
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
    proposals: Arc<CreditFacilityProposals<Perms, E>>,
    pending_credit_facilities: Arc<PendingCreditFacilities<Perms, E>>,
    audit: Arc<Perms::Audit>,
    governance: Arc<Governance<Perms, E>>,
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
        proposals: Arc<CreditFacilityProposals<Perms, E>>,
        pending_credit_facilities: Arc<PendingCreditFacilities<Perms, E>>,
        audit: Arc<Perms::Audit>,
        governance: Arc<Governance<Perms, E>>,
    ) -> Self {
        Self {
            proposals,
            pending_credit_facilities,
            audit,
            governance,
        }
    }

    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    #[instrument(name = "credit_facility.approval.execute", skip(self))]
    pub async fn execute(
        &self,
        id: impl es_entity::RetryableInto<CreditFacilityProposalId>,
        approved: bool,
    ) -> Result<Option<CreditFacilityProposal>, CoreCreditError> {
        let proposal = self
            .pending_credit_facilities
            .transition_from_proposal(id.into(), approved)
            .await?;

        Ok(proposal)
    }
}
