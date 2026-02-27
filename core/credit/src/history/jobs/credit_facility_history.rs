use tracing::{Span, instrument};

use std::sync::Arc;

use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use core_credit_collateral::public::CoreCreditCollateralEvent;
use job::JobType;

use crate::{CoreCreditCollectionEvent, CoreCreditEvent, primitives::CreditFacilityId};

use super::super::repo::HistoryRepo;

pub const HISTORY_PROJECTION: JobType = JobType::new("outbox.credit-facility-history-projection");

pub struct HistoryProjectionHandler<
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>,
> {
    repo: Arc<HistoryRepo>,
    _phantom: std::marker::PhantomData<E>,
}

impl<E> HistoryProjectionHandler<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub fn new(repo: Arc<HistoryRepo>) -> Self {
        Self {
            repo,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E> OutboxEventHandler<E> for HistoryProjectionHandler<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    #[instrument(name = "outbox.core_credit.history_projection_job.process_message", parent = None, skip(self, op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use CoreCreditEvent::*;

        let db = op.tx_mut();

        match event.as_event() {
            Some(e @ FacilityProposalCreated { entity }) => {
                self.handle_credit_event(db, event, e, entity.id).await?;
            }
            Some(e @ FacilityProposalConcluded { entity })
                if entity.status == crate::primitives::CreditFacilityProposalStatus::Approved =>
            {
                self.handle_credit_event(db, event, e, entity.id).await?;
            }
            Some(e @ PendingCreditFacilityCollateralizationChanged { entity }) => {
                self.handle_credit_event(db, event, e, entity.id).await?;
            }
            Some(e @ FacilityActivated { entity }) => {
                self.handle_credit_event(db, event, e, entity.id).await?;
            }
            Some(e @ FacilityCompleted { entity }) => {
                self.handle_credit_event(db, event, e, entity.id).await?;
            }
            Some(e @ DisbursalSettled { entity }) => {
                self.handle_credit_event(db, event, e, entity.credit_facility_id)
                    .await?;
            }
            Some(e @ AccrualPosted { entity }) => {
                self.handle_credit_event(db, event, e, entity.credit_facility_id)
                    .await?;
            }
            Some(e @ PartialLiquidationInitiated { entity }) => {
                self.handle_credit_event(db, event, e, entity.id).await?;
            }
            Some(e @ FacilityCollateralizationChanged { entity }) => {
                self.handle_credit_event(db, event, e, entity.id).await?;
            }

            _ => {}
        }

        use CoreCreditCollateralEvent::*;
        match event.as_event() {
            Some(e @ CollateralUpdated { entity }) => {
                self.handle_collateral_event(db, event, e, entity.secured_loan_id)
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
                self.handle_collateral_event(db, event, e, *id).await?;
            }
            _ => {}
        }

        if let Some(e @ CoreCreditCollectionEvent::PaymentAllocationCreated { entity }) =
            event.as_event()
        {
            let id: CreditFacilityId = entity.beneficiary_id.into();
            self.handle_collection_event(db, event, e, id).await?;
        }

        Ok(())
    }
}

impl<E> HistoryProjectionHandler<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    async fn handle_credit_event(
        &self,
        db: &mut sqlx::PgTransaction<'_>,
        message: &PersistentOutboxEvent<E>,
        event: &CoreCreditEvent,
        id: impl Into<CreditFacilityId>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let id = id.into();
        message.inject_trace_parent();
        Span::current().record("handled", true);
        Span::current().record("event_type", event.as_ref());
        let mut history = self.repo.load(id).await?;
        history.process_credit_event(event, message.recorded_at);
        self.repo.persist_in_tx(db, id, history).await?;
        Ok(())
    }

    async fn handle_collateral_event(
        &self,
        db: &mut sqlx::PgTransaction<'_>,
        message: &PersistentOutboxEvent<E>,
        event: &CoreCreditCollateralEvent,
        id: impl Into<CreditFacilityId>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let id = id.into();
        message.inject_trace_parent();
        Span::current().record("handled", true);
        Span::current().record("event_type", event.as_ref());
        let mut history = self.repo.load(id).await?;
        history.process_collateral_event(event, message.recorded_at);
        self.repo.persist_in_tx(db, id, history).await?;
        Ok(())
    }

    async fn handle_collection_event(
        &self,
        db: &mut sqlx::PgTransaction<'_>,
        message: &PersistentOutboxEvent<E>,
        event: &CoreCreditCollectionEvent,
        id: impl Into<CreditFacilityId>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let id = id.into();
        message.inject_trace_parent();
        Span::current().record("handled", true);
        Span::current().record("event_type", event.as_ref());
        let mut history = self.repo.load(id).await?;
        history.process_collection_event(event);
        self.repo.persist_in_tx(db, id, history).await?;
        Ok(())
    }
}
