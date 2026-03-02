mod credit_facility_proposal_approval;
mod execute_approve_credit_facility_proposal;

pub use credit_facility_proposal_approval::*;
pub use execute_approve_credit_facility_proposal::*;

use std::sync::Arc;

use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_credit_collateral::{
    CoreCreditCollateralAction, CoreCreditCollateralObject, public::CoreCreditCollateralEvent,
};
use governance::{
    ApprovalProcessType, Governance, GovernanceAction, GovernanceEvent, GovernanceObject,
};
use obix::out::OutboxEventMarker;

use crate::{
    CoreCreditAction, CoreCreditCollectionEvent, CoreCreditEvent, CoreCreditObject,
    CreditFacilityProposal, CreditFacilityProposalId, CreditFacilityProposals,
    PendingCreditFacilities, error::CoreCreditError,
};

use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_price::CorePriceEvent;
pub const APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS: ApprovalProcessType =
    ApprovalProcessType::new("credit-facility-proposal");

pub struct ApproveCreditFacilityProposal<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
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
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
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

    #[record_error_severity]
    #[instrument(name = "credit_facility.approval.execute_in_op",
        skip(self, op, credit_facility_proposal_id),
        fields(credit_facility_proposal_id = tracing::field::Empty))
     ]
    pub async fn execute_approve_credit_facility_proposal_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        credit_facility_proposal_id: CreditFacilityProposalId,
        approved: bool,
    ) -> Result<Option<CreditFacilityProposal>, CoreCreditError> {
        tracing::Span::current().record(
            "credit_facility_proposal_id",
            tracing::field::display(&credit_facility_proposal_id),
        );
        let proposal = self
            .pending_credit_facilities
            .transition_from_proposal_in_op(op, credit_facility_proposal_id, approved)
            .await?;

        Ok(proposal)
    }
}
