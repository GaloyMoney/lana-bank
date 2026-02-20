mod job;

use std::sync::Arc;

use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{
    ApprovalProcessType, Governance, GovernanceAction, GovernanceEvent, GovernanceObject,
};
use obix::out::OutboxEventMarker;

use crate::{
    CoreCreditAction, CoreCreditCollectionEvent, CoreCreditEvent, CoreCreditObject,
    CreditFacilityProposal, CreditFacilityProposalId, CreditFacilityProposals,
    PendingCreditFacilities, collateral::ledger::CollateralLedgerOps, error::CoreCreditError,
    ledger::CreditLedgerOps,
};

use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_price::CorePriceEvent;

pub use job::*;
pub const APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS: ApprovalProcessType =
    ApprovalProcessType::new("credit-facility-proposal");

pub struct ApproveCreditFacilityProposal<Perms, E, L, CL>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
    L: CreditLedgerOps,
    CL: CollateralLedgerOps,
{
    proposals: Arc<CreditFacilityProposals<Perms, E>>,
    pending_credit_facilities: Arc<PendingCreditFacilities<Perms, E, L, CL>>,
    audit: Arc<Perms::Audit>,
    governance: Arc<Governance<Perms, E>>,
}

impl<Perms, E, L, CL> Clone for ApproveCreditFacilityProposal<Perms, E, L, CL>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
    L: CreditLedgerOps,
    CL: CollateralLedgerOps,
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

impl<Perms, E, L, CL> ApproveCreditFacilityProposal<Perms, E, L, CL>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<crate::CoreCreditCollectionAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<crate::CoreCreditCollectionObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
    L: CreditLedgerOps,
    CL: CollateralLedgerOps,
{
    pub fn new(
        proposals: Arc<CreditFacilityProposals<Perms, E>>,
        pending_credit_facilities: Arc<PendingCreditFacilities<Perms, E, L, CL>>,
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

    #[record_error_severity]
    #[instrument(name = "credit_facility.approval.execute",
        skip(self, credit_facility_proposal_id),
        fields(credit_facility_proposal_id = tracing::field::Empty))
     ]
    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    pub async fn execute_approve_credit_facility_proposal(
        &self,
        credit_facility_proposal_id: impl es_entity::RetryableInto<CreditFacilityProposalId>,
        approved: bool,
    ) -> Result<Option<CreditFacilityProposal>, CoreCreditError> {
        let id = credit_facility_proposal_id.into();
        tracing::Span::current()
            .record("credit_facility_proposal_id", tracing::field::display(&id));
        let proposal = self
            .pending_credit_facilities
            .transition_from_proposal(id, approved)
            .await?;

        Ok(proposal)
    }
}
