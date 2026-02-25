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
    CoreCreditAction, CoreCreditCollectionEvent, CoreCreditEvent, CoreCreditObject,
    PendingCreditFacilityCollateralizationState,
    collateral::{
        CoreCreditCollateralAction, CoreCreditCollateralObject, public::CoreCreditCollateralEvent,
    },
};

use super::ActivateCreditFacility;

pub const CREDIT_FACILITY_ACTIVATE: JobType = JobType::new("outbox.credit-facility-activation");

pub(crate) struct CreditFacilityActivationHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    process: ActivateCreditFacility<Perms, E>,
}

impl<Perms, E> CreditFacilityActivationHandler<Perms, E>
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
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    pub fn new(process: &ActivateCreditFacility<Perms, E>) -> Self {
        Self {
            process: process.clone(),
        }
    }
}

impl<Perms, E> OutboxEventHandler<E> for CreditFacilityActivationHandler<Perms, E>
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
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    #[instrument(name = "core_credit.credit_facility_activation_job.process_message", parent = None, skip(self, _op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty, pending_credit_facility_id = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        _op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use CoreCreditEvent::*;

        if let Some(e @ PendingCreditFacilityCollateralizationChanged { entity }) = event.as_event()
            && entity.collateralization.state
                == PendingCreditFacilityCollateralizationState::FullyCollateralized
        {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record(
                "pending_credit_facility_id",
                tracing::field::display(entity.id),
            );
            Span::current().record("event_type", e.as_ref());

            self.process
                .execute_activate_credit_facility(entity.id)
                .await?;
        }
        Ok(())
    }
}
