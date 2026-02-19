use tracing::{Span, instrument};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_price::CorePriceEvent;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::JobType;

use crate::{CoreCreditAction, CoreCreditEvent, CoreCreditObject};
use core_credit_collection::CoreCreditCollectionEvent;

use super::ApproveCreditFacilityProposal;

pub const CREDIT_FACILITY_PROPOSAL_APPROVE_JOB: JobType =
    JobType::new("outbox.credit-facility-proposal-approval");

pub(crate) struct CreditFacilityProposalApprovalHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    process: ApproveCreditFacilityProposal<Perms, E>,
}

impl<Perms, E> CreditFacilityProposalApprovalHandler<Perms, E>
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
{
    pub fn new(process: &ApproveCreditFacilityProposal<Perms, E>) -> Self {
        Self {
            process: process.clone(),
        }
    }
}

impl<Perms, E> OutboxEventHandler<E> for CreditFacilityProposalApprovalHandler<Perms, E>
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
{
    #[instrument(name = "core_credit.credit_facility_proposal_approval_job.process_message", parent = None, skip(self, _op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty, process_type = tracing::field::Empty, credit_facility_proposal_id = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        _op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match event.as_event() {
            Some(e @ GovernanceEvent::ApprovalProcessConcluded { entity })
                if entity.process_type == super::APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS =>
            {
                event.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", e.as_ref());
                Span::current().record(
                    "credit_facility_proposal_id",
                    tracing::field::display(entity.id),
                );
                Span::current().record("process_type", entity.process_type.to_string());
                self.process
                    .execute_approve_credit_facility_proposal(
                        entity.id,
                        entity.status.is_approved(),
                    )
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}
