use tracing::{Span, instrument};

use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use core_credit_collection::{PublicObligation, PublicPaymentAllocation};

use crate::{CoreCreditCollectionEvent, CoreCreditEvent, primitives::CreditFacilityId};

use super::update_repayment_plan::UpdateRepaymentPlanConfig;

pub const REPAYMENT_PLAN_PROJECTION: JobType =
    JobType::new("outbox.credit-facility-repayment-plan-projection");

pub struct RepaymentPlanProjectionHandler {
    update_repayment_plan: JobSpawner<UpdateRepaymentPlanConfig>,
}

impl RepaymentPlanProjectionHandler {
    pub fn new(update_repayment_plan: JobSpawner<UpdateRepaymentPlanConfig>) -> Self {
        Self {
            update_repayment_plan,
        }
    }
}

impl<E> OutboxEventHandler<E> for RepaymentPlanProjectionHandler
where
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    #[instrument(name = "outbox.core_credit.repayment_plan_projection_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let sequence = event.sequence;

        use CoreCreditEvent::*;

        match event.as_event() {
            Some(e @ FacilityProposalCreated { entity }) => {
                self.spawn_credit_event_in_op(op, event, e, entity.id.into(), sequence)
                    .await?;
            }
            Some(e @ FacilityProposalConcluded { entity })
                if entity.status == crate::primitives::CreditFacilityProposalStatus::Approved =>
            {
                self.spawn_credit_event_in_op(op, event, e, entity.id.into(), sequence)
                    .await?;
            }
            Some(e @ FacilityActivated { entity })
            | Some(e @ FacilityCompleted { entity })
            | Some(e @ PartialLiquidationInitiated { entity }) => {
                self.spawn_credit_event_in_op(op, event, e, entity.id, sequence)
                    .await?;
            }
            Some(e @ DisbursalSettled { entity }) => {
                self.spawn_credit_event_in_op(op, event, e, entity.credit_facility_id, sequence)
                    .await?;
            }
            Some(e @ AccrualPosted { entity }) => {
                self.spawn_credit_event_in_op(op, event, e, entity.credit_facility_id, sequence)
                    .await?;
            }
            Some(e @ FacilityCollateralizationChanged { entity }) => {
                self.spawn_credit_event_in_op(op, event, e, entity.id, sequence)
                    .await?;
            }
            _ => {}
        }

        use CoreCreditCollectionEvent::*;

        match event.as_event() {
            Some(
                e @ PaymentAllocationCreated {
                    entity: PublicPaymentAllocation { beneficiary_id, .. },
                },
            )
            | Some(
                e @ ObligationCreated {
                    entity: PublicObligation { beneficiary_id, .. },
                },
            )
            | Some(
                e @ ObligationDue {
                    entity: PublicObligation { beneficiary_id, .. },
                },
            )
            | Some(
                e @ ObligationOverdue {
                    entity: PublicObligation { beneficiary_id, .. },
                },
            )
            | Some(
                e @ ObligationDefaulted {
                    entity: PublicObligation { beneficiary_id, .. },
                },
            )
            | Some(
                e @ ObligationCompleted {
                    entity: PublicObligation { beneficiary_id, .. },
                },
            ) => {
                self.spawn_collection_event_in_op(op, event, e, (*beneficiary_id).into(), sequence)
                    .await?;
            }
            _ => {}
        }

        Ok(())
    }
}

impl RepaymentPlanProjectionHandler {
    async fn spawn_credit_event_in_op<E>(
        &self,
        op: &mut es_entity::DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
        event: &CoreCreditEvent,
        facility_id: CreditFacilityId,
        sequence: obix::EventSequence,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
    {
        message.inject_trace_parent();
        Span::current().record("handled", true);
        Span::current().record("event_type", event.as_ref());
        self.update_repayment_plan
            .spawn_with_queue_id_in_op(
                op,
                JobId::new(),
                UpdateRepaymentPlanConfig::CreditEvent {
                    facility_id,
                    sequence,
                    recorded_at: message.recorded_at,
                    event: serde_json::to_value(event)?,
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
        sequence: obix::EventSequence,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
    {
        message.inject_trace_parent();
        Span::current().record("handled", true);
        Span::current().record("event_type", event.as_ref());
        self.update_repayment_plan
            .spawn_with_queue_id_in_op(
                op,
                JobId::new(),
                UpdateRepaymentPlanConfig::CollectionEvent {
                    facility_id,
                    sequence,
                    event: serde_json::to_value(event)?,
                },
                facility_id.to_string(),
            )
            .await?;
        Ok(())
    }
}
