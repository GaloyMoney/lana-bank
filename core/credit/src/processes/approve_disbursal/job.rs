use tracing::{Span, instrument};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_price::CorePriceEvent;
use core_time_events::CoreTimeEvent;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::JobType;

use crate::{
    CoreCreditAction, CoreCreditCollectionAction, CoreCreditCollectionEvent,
    CoreCreditCollectionObject, CoreCreditEvent, CoreCreditObject,
    collateral::{
        CoreCreditCollateralAction, CoreCreditCollateralObject, public::CoreCreditCollateralEvent,
    },
};

use super::ApproveDisbursal;

pub const DISBURSAL_APPROVE_JOB: JobType = JobType::new("outbox.disbursal-approval");

pub(crate) struct DisbursalApprovalHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    process: ApproveDisbursal<Perms, E>,
}

impl<Perms, E> DisbursalApprovalHandler<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    pub fn new(process: &ApproveDisbursal<Perms, E>) -> Self {
        Self {
            process: process.clone(),
        }
    }
}

impl<Perms, E> OutboxEventHandler<E> for DisbursalApprovalHandler<Perms, E>
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
        + OutboxEventMarker<CorePriceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    #[instrument(name = "core_credit.disbursal_approval_job.process_message", parent = None, skip(self, _op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty, process_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        _op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match event.as_event() {
            Some(e @ GovernanceEvent::ApprovalProcessConcluded { entity })
                if entity.process_type == super::APPROVE_DISBURSAL_PROCESS =>
            {
                event.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", e.as_ref());
                Span::current().record("process_type", entity.process_type.to_string());
                self.process
                    .execute_approve_disbursal(entity.id, entity.status.is_approved())
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}
