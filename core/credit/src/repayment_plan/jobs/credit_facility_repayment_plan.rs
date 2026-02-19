use tracing::{Span, instrument};

use std::sync::Arc;

use es_entity::AtomicOperation;
use obix::EventSequence;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::JobType;

use core_credit_collection::{PublicObligation, PublicPaymentAllocation};

use crate::{CoreCreditCollectionEvent, CoreCreditEvent, repayment_plan::*};

pub const REPAYMENT_PLAN_PROJECTION: JobType =
    JobType::new("outbox.credit-facility-repayment-plan-projection");

pub struct RepaymentPlanProjectionHandler<
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
> {
    repo: Arc<RepaymentPlanRepo>,
    _phantom: std::marker::PhantomData<E>,
}

impl<E> RepaymentPlanProjectionHandler<E>
where
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub fn new(repo: Arc<RepaymentPlanRepo>) -> Self {
        Self {
            repo,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E> OutboxEventHandler<E> for RepaymentPlanProjectionHandler<E>
where
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    #[instrument(name = "outbox.core_credit.repayment_plan_projection_job.process_message", parent = None, skip(self, op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let sequence = event.sequence;
        let clock = op.clock().clone();
        let db = op.tx_mut();

        use CoreCreditEvent::*;

        match event.as_event() {
            Some(e @ FacilityProposalCreated { entity }) => {
                self.handle_credit_event(db, event, e, entity.id, sequence, &clock)
                    .await?;
            }
            Some(e @ FacilityProposalConcluded { entity })
                if entity.status == crate::primitives::CreditFacilityProposalStatus::Approved =>
            {
                self.handle_credit_event(db, event, e, entity.id, sequence, &clock)
                    .await?;
            }
            Some(e @ FacilityActivated { entity })
            | Some(e @ FacilityCompleted { entity })
            | Some(e @ PartialLiquidationInitiated { entity }) => {
                self.handle_credit_event(db, event, e, entity.id, sequence, &clock)
                    .await?;
            }
            Some(e @ DisbursalSettled { entity }) => {
                self.handle_credit_event(db, event, e, entity.credit_facility_id, sequence, &clock)
                    .await?;
            }
            Some(e @ AccrualPosted { entity }) => {
                self.handle_credit_event(db, event, e, entity.credit_facility_id, sequence, &clock)
                    .await?;
            }
            Some(e @ FacilityCollateralUpdated { entity }) => {
                self.handle_credit_event(db, event, e, entity.secured_loan_id, sequence, &clock)
                    .await?;
            }
            Some(e @ FacilityCollateralizationChanged { entity }) => {
                self.handle_credit_event(db, event, e, entity.id, sequence, &clock)
                    .await?;
            }
            Some(
                e @ PartialLiquidationCompleted {
                    credit_facility_id: id,
                    ..
                },
            ) => {
                self.handle_credit_event(db, event, e, *id, sequence, &clock)
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
                self.handle_collection_event(db, event, e, *beneficiary_id, sequence, &clock)
                    .await?;
            }
            _ => {}
        }

        Ok(())
    }
}

impl<E> RepaymentPlanProjectionHandler<E>
where
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    async fn handle_credit_event(
        &self,
        db: &mut sqlx::PgTransaction<'_>,
        message: &PersistentOutboxEvent<E>,
        event: &CoreCreditEvent,
        id: impl Into<crate::primitives::CreditFacilityId>,
        sequence: EventSequence,
        clock: &es_entity::clock::ClockHandle,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let id = id.into();
        message.inject_trace_parent();
        Span::current().record("handled", true);
        Span::current().record("event_type", event.as_ref());
        let mut repayment_plan = self.repo.load(id).await?;
        repayment_plan.process_credit_event(sequence, event, clock.now(), message.recorded_at);
        self.repo.persist_in_tx(db, id, repayment_plan).await?;
        Ok(())
    }

    async fn handle_collection_event(
        &self,
        db: &mut sqlx::PgTransaction<'_>,
        message: &PersistentOutboxEvent<E>,
        event: &CoreCreditCollectionEvent,
        id: impl Into<crate::primitives::CreditFacilityId>,
        sequence: EventSequence,
        clock: &es_entity::clock::ClockHandle,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let id = id.into();
        message.inject_trace_parent();
        Span::current().record("handled", true);
        Span::current().record("event_type", event.as_ref());
        let mut repayment_plan = self.repo.load(id).await?;
        repayment_plan.process_collection_event(sequence, event, clock.now());
        self.repo.persist_in_tx(db, id, repayment_plan).await?;
        Ok(())
    }
}
