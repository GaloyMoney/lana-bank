use tracing::{Span, instrument};

use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use crate::{
    CoreCreditCollectionEvent, CoreCreditEvent, collateral::public::CoreCreditCollateralEvent,
    primitives::CreditFacilityId,
};

use super::{
    update_collateral_history::UpdateCollateralHistoryConfig,
    update_collection_history::UpdateCollectionHistoryConfig,
    update_credit_history::UpdateCreditHistoryConfig,
};

pub const HISTORY_PROJECTION: JobType = JobType::new("outbox.credit-facility-history-projection");

pub struct HistoryProjectionHandler {
    update_credit_history: JobSpawner<UpdateCreditHistoryConfig>,
    update_collateral_history: JobSpawner<UpdateCollateralHistoryConfig>,
    update_collection_history: JobSpawner<UpdateCollectionHistoryConfig>,
}

impl HistoryProjectionHandler {
    pub fn new(
        update_credit_history: JobSpawner<UpdateCreditHistoryConfig>,
        update_collateral_history: JobSpawner<UpdateCollateralHistoryConfig>,
        update_collection_history: JobSpawner<UpdateCollectionHistoryConfig>,
    ) -> Self {
        Self {
            update_credit_history,
            update_collateral_history,
            update_collection_history,
        }
    }
}

impl<E> OutboxEventHandler<E> for HistoryProjectionHandler
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    #[instrument(name = "outbox.core_credit.history_projection_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use CoreCreditEvent::*;

        match event.as_event() {
            Some(e @ FacilityProposalCreated { entity }) => {
                self.spawn_credit_event_in_op(op, event, e, entity.id.into())
                    .await?;
            }
            Some(e @ FacilityProposalConcluded { entity })
                if entity.status == crate::primitives::CreditFacilityProposalStatus::Approved =>
            {
                self.spawn_credit_event_in_op(op, event, e, entity.id.into())
                    .await?;
            }
            Some(e @ PendingCreditFacilityCollateralizationChanged { entity }) => {
                self.spawn_credit_event_in_op(op, event, e, entity.id.into())
                    .await?;
            }
            Some(e @ FacilityActivated { entity }) => {
                self.spawn_credit_event_in_op(op, event, e, entity.id)
                    .await?;
            }
            Some(e @ FacilityCompleted { entity }) => {
                self.spawn_credit_event_in_op(op, event, e, entity.id)
                    .await?;
            }
            Some(e @ DisbursalSettled { entity }) => {
                self.spawn_credit_event_in_op(op, event, e, entity.credit_facility_id)
                    .await?;
            }
            Some(e @ AccrualPosted { entity }) => {
                self.spawn_credit_event_in_op(op, event, e, entity.credit_facility_id)
                    .await?;
            }
            Some(e @ PartialLiquidationInitiated { entity }) => {
                self.spawn_credit_event_in_op(op, event, e, entity.id)
                    .await?;
            }
            Some(e @ FacilityCollateralizationChanged { entity }) => {
                self.spawn_credit_event_in_op(op, event, e, entity.id)
                    .await?;
            }

            _ => {}
        }

        use CoreCreditCollateralEvent::*;
        match event.as_event() {
            Some(e @ CollateralUpdated { entity }) => {
                self.spawn_collateral_event_in_op(op, event, e, entity.secured_loan_id.into())
                    .await?;
            }
            Some(
                e @ LiquidationCompleted {
                    secured_loan_id: id,
                    ..
                },
            )
            | Some(
                e @ LiquidationProceedsReceived {
                    secured_loan_id: id,
                    ..
                },
            )
            | Some(
                e @ LiquidationCollateralSentOut {
                    secured_loan_id: id,
                    ..
                },
            ) => {
                self.spawn_collateral_event_in_op(op, event, e, (*id).into())
                    .await?;
            }
            _ => {}
        }

        if let Some(e @ CoreCreditCollectionEvent::PaymentAllocationCreated { entity }) =
            event.as_event()
        {
            let facility_id: CreditFacilityId = entity.beneficiary_id.into();
            self.spawn_collection_event_in_op(op, event, e, facility_id)
                .await?;
        }

        Ok(())
    }
}

impl HistoryProjectionHandler {
    async fn spawn_credit_event_in_op<E>(
        &self,
        op: &mut es_entity::DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
        event: &CoreCreditEvent,
        facility_id: CreditFacilityId,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        E: OutboxEventMarker<CoreCreditEvent>
            + OutboxEventMarker<CoreCreditCollateralEvent>
            + OutboxEventMarker<CoreCreditCollectionEvent>,
    {
        message.inject_trace_parent();
        Span::current().record("handled", true);
        Span::current().record("event_type", event.as_ref());
        self.update_credit_history
            .spawn_with_queue_id_in_op(
                op,
                JobId::new(),
                UpdateCreditHistoryConfig {
                    facility_id,
                    recorded_at: message.recorded_at,
                    event: event.clone(),
                },
                facility_id.to_string(),
            )
            .await?;
        Ok(())
    }

    async fn spawn_collateral_event_in_op<E>(
        &self,
        op: &mut es_entity::DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
        event: &CoreCreditCollateralEvent,
        facility_id: CreditFacilityId,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        E: OutboxEventMarker<CoreCreditEvent>
            + OutboxEventMarker<CoreCreditCollateralEvent>
            + OutboxEventMarker<CoreCreditCollectionEvent>,
    {
        message.inject_trace_parent();
        Span::current().record("handled", true);
        Span::current().record("event_type", event.as_ref());
        self.update_collateral_history
            .spawn_with_queue_id_in_op(
                op,
                JobId::new(),
                UpdateCollateralHistoryConfig {
                    facility_id,
                    recorded_at: message.recorded_at,
                    event: event.clone(),
                },
                facility_id.to_string(),
            )
            .await?;
        Ok(())
    }

    async fn spawn_collection_event_in_op<E>(
        &self,
        op: &mut es_entity::DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
        event: &CoreCreditCollectionEvent,
        facility_id: CreditFacilityId,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        E: OutboxEventMarker<CoreCreditEvent>
            + OutboxEventMarker<CoreCreditCollateralEvent>
            + OutboxEventMarker<CoreCreditCollectionEvent>,
    {
        message.inject_trace_parent();
        Span::current().record("handled", true);
        Span::current().record("event_type", event.as_ref());
        self.update_collection_history
            .spawn_with_queue_id_in_op(
                op,
                JobId::new(),
                UpdateCollectionHistoryConfig {
                    facility_id,
                    event: event.clone(),
                },
                facility_id.to_string(),
            )
            .await?;
        Ok(())
    }
}
