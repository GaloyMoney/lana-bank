use tracing::{Span, instrument};

use std::{marker::PhantomData, sync::Arc};

use authz::PermissionCheck;
use governance::GovernanceEvent;
use obix::out::{
    EphemeralOutboxEvent, OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent,
};

use job::{JobId, JobSpawner, JobType};

use core_credit_collateral::{Collaterals, CoreCreditCollateralEvent};
use core_credit_collection::{PublicObligation, PublicPaymentAllocation};
use core_custody::CoreCustodyEvent;
use core_price::CorePriceEvent;

use crate::{CoreCreditCollectionEvent, CoreCreditEvent, ledger::CreditLedger};

use super::collect_facilities_for_collateralization::{
    CollectFacilitiesForCollateralizationJobConfig, CollectFacilitiesForCollateralizationJobSpawner,
};
use super::update_collateralization::UpdateCollateralizationConfig;

pub const CREDIT_FACILITY_COLLATERALIZATION_FROM_EVENTS_JOB: JobType =
    JobType::new("outbox.credit-facility-collateralization");

pub struct CreditFacilityCollateralizationFromEventsHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    update_collateralization: JobSpawner<UpdateCollateralizationConfig>,
    collect_for_collateralization: CollectFacilitiesForCollateralizationJobSpawner,
    _phantom: PhantomData<(Perms, E)>,
}

impl<Perms, E> CreditFacilityCollateralizationFromEventsHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(
        update_collateralization: JobSpawner<UpdateCollateralizationConfig>,
        collect_for_collateralization: CollectFacilitiesForCollateralizationJobSpawner,
        _collaterals: Arc<Collaterals<Perms, E>>,
        _ledger: Arc<CreditLedger>,
        _authz: Arc<Perms>,
    ) -> Self {
        Self {
            update_collateralization,
            collect_for_collateralization,
            _phantom: PhantomData,
        }
    }
}

impl<Perms, E> OutboxEventHandler<E> for CreditFacilityCollateralizationFromEventsHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[instrument(name = "core_credit.collateralization_job.process_persistent_message", parent = None, skip(self, op, message), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty, credit_facility_id = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(event @ CoreCreditCollateralEvent::CollateralUpdated { entity }) =
            message.as_event()
        {
            message.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());
            Span::current().record(
                "credit_facility_id",
                tracing::field::display(entity.secured_loan_id),
            );

            self.update_collateralization
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    UpdateCollateralizationConfig {
                        credit_facility_id: entity.secured_loan_id.into(),
                    },
                    entity.secured_loan_id.to_string(),
                )
                .await?;
        }

        if let Some(
            event @ (CoreCreditCollectionEvent::ObligationCreated {
                entity: PublicObligation { beneficiary_id, .. },
            }
            | CoreCreditCollectionEvent::PaymentAllocationCreated {
                entity: PublicPaymentAllocation { beneficiary_id, .. },
            }),
        ) = message.as_event()
        {
            message.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());
            Span::current().record(
                "credit_facility_id",
                tracing::field::display(beneficiary_id),
            );

            self.update_collateralization
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    UpdateCollateralizationConfig {
                        credit_facility_id: (*beneficiary_id).into(),
                    },
                    beneficiary_id.to_string(),
                )
                .await?;
        }

        Ok(())
    }

    #[instrument(name = "core_credit.credit_facility_collateralization_job.process_ephemeral_message", parent = None, skip(self, message), fields(handled = false, event_type = tracing::field::Empty))]
    #[allow(clippy::single_match)]
    async fn handle_ephemeral(
        &self,
        message: &EphemeralOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match message.payload.as_event() {
            Some(CorePriceEvent::PriceUpdated { price, .. }) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", tracing::field::display(&message.event_type));

                self.collect_for_collateralization
                    .spawn(
                        JobId::new(),
                        CollectFacilitiesForCollateralizationJobConfig { price: *price },
                    )
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}
