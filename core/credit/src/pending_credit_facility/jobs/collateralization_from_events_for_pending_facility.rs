use tracing::{Span, instrument};

use governance::GovernanceEvent;
use obix::out::{
    EphemeralOutboxEvent, OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent,
};

use job::{JobId, JobSpawner, JobType};

use core_credit_collateral::public::CoreCreditCollateralEvent;
use core_credit_collection::CoreCreditCollectionEvent;
use core_custody::CoreCustodyEvent;
use core_price::CorePriceEvent;

use crate::CoreCreditEvent;

use super::update_pending_collateralization::UpdatePendingCollateralizationConfig;
use super::update_pending_collateralization_from_price::{
    PENDING_PRICE_SWEEP_QUEUE_ID, UpdatePendingCollateralizationFromPriceConfig,
};

pub const PENDING_CREDIT_FACILITY_COLLATERALIZATION_FROM_EVENTS_JOB: JobType =
    JobType::new("outbox.pending-credit-facility-collateralization-from-events");

pub struct PendingCreditFacilityCollateralizationFromEventsHandler {
    update_pending_collateralization: JobSpawner<UpdatePendingCollateralizationConfig>,
    update_pending_collateralization_from_price:
        JobSpawner<UpdatePendingCollateralizationFromPriceConfig>,
}

impl PendingCreditFacilityCollateralizationFromEventsHandler {
    pub fn new(
        update_pending_collateralization: JobSpawner<UpdatePendingCollateralizationConfig>,
        update_pending_collateralization_from_price: JobSpawner<
            UpdatePendingCollateralizationFromPriceConfig,
        >,
    ) -> Self {
        Self {
            update_pending_collateralization,
            update_pending_collateralization_from_price,
        }
    }
}

impl<E> OutboxEventHandler<E> for PendingCreditFacilityCollateralizationFromEventsHandler
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[instrument(name = "core_credit.pending_credit_facility_collateralization_job.process_persistent_message", parent = None, skip(self, op, message), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty, pending_credit_facility_id = tracing::field::Empty))]
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
                "pending_credit_facility_id",
                tracing::field::display(entity.secured_loan_id),
            );

            self.update_pending_collateralization
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    UpdatePendingCollateralizationConfig {
                        pending_credit_facility_id: entity.secured_loan_id.into(),
                    },
                    entity.secured_loan_id.to_string(),
                )
                .await?;
        }
        Ok(())
    }

    #[instrument(name = "core_credit.pending_credit_facility_collateralization_job.process_ephemeral_message", parent = None, skip(self, message), fields(handled = false, event_type = tracing::field::Empty))]
    async fn handle_ephemeral(
        &self,
        message: &EphemeralOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(CorePriceEvent::PriceUpdated { price, .. }) = message.payload.as_event() {
            message.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", tracing::field::display(&message.event_type));

            self.update_pending_collateralization_from_price
                .spawn_with_queue_id(
                    JobId::new(),
                    UpdatePendingCollateralizationFromPriceConfig { price: *price },
                    PENDING_PRICE_SWEEP_QUEUE_ID,
                )
                .await?;
        }
        Ok(())
    }
}
