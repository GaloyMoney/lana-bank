use obix::out::{Outbox, OutboxEventMarker};
use tracing::instrument;
use tracing_macros::record_error_severity;

use crate::{
    event::*,
    obligation::{Obligation, ObligationEvent, error::ObligationError},
    payment::{Payment, PaymentEvent, error::PaymentError},
    payment_allocation::{
        PaymentAllocation, PaymentAllocationEvent, error::PaymentAllocationError,
    },
};

pub struct CollectionsPublisher<E>
where
    E: OutboxEventMarker<CoreCreditCollectionsEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for CollectionsPublisher<E>
where
    E: OutboxEventMarker<CoreCreditCollectionsEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> CollectionsPublisher<E>
where
    E: OutboxEventMarker<CoreCreditCollectionsEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    #[record_error_severity]
    #[instrument(name = "collections.publisher.publish_payment", skip_all)]
    pub async fn publish_payment(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        _entity: &Payment,
        new_events: es_entity::LastPersisted<'_, PaymentEvent>,
    ) -> Result<(), PaymentError> {
        use PaymentEvent::*;
        let publish_events = new_events
            .map(|event| match &event.event {
                Initialized {
                    id,
                    credit_facility_id,
                    amount,
                    effective,
                    ..
                } => CoreCreditCollectionsEvent::PaymentReceived {
                    payment_id: *id,
                    credit_facility_id: *credit_facility_id,
                    amount: *amount,
                    recorded_at: event.recorded_at,
                    effective: *effective,
                },
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "collections.publisher.publish_payment_allocation", skip_all)]
    pub async fn publish_payment_allocation(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &PaymentAllocation,
        new_events: es_entity::LastPersisted<'_, PaymentAllocationEvent>,
    ) -> Result<(), PaymentAllocationError> {
        use PaymentAllocationEvent::*;
        let publish_events = new_events
            .map(|event| match &event.event {
                Initialized {
                    id,
                    obligation_id,
                    obligation_type,
                    amount,
                    effective,
                    ..
                } => CoreCreditCollectionsEvent::PaymentAllocated {
                    credit_facility_id: entity.credit_facility_id,
                    obligation_id: *obligation_id,
                    obligation_type: *obligation_type,
                    allocation_id: *id,
                    amount: *amount,
                    recorded_at: event.recorded_at,
                    effective: *effective,
                },
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "collections.publisher.publish_obligation", skip_all)]
    pub async fn publish_obligation(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Obligation,
        new_events: es_entity::LastPersisted<'_, ObligationEvent>,
    ) -> Result<(), ObligationError> {
        use ObligationEvent::*;

        let dates = entity.lifecycle_dates();
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Initialized { effective, .. } => {
                    Some(CoreCreditCollectionsEvent::ObligationCreated {
                        id: entity.id,
                        obligation_type: entity.obligation_type,
                        credit_facility_id: entity.credit_facility_id,
                        amount: entity.initial_amount,
                        due_at: dates.due,
                        overdue_at: dates.overdue,
                        defaulted_at: dates.defaulted,
                        recorded_at: event.recorded_at,
                        effective: *effective,
                    })
                }
                DueRecorded {
                    due_amount: amount, ..
                } => Some(CoreCreditCollectionsEvent::ObligationDue {
                    id: entity.id,
                    credit_facility_id: entity.credit_facility_id,
                    obligation_type: entity.obligation_type,
                    amount: *amount,
                }),
                OverdueRecorded {
                    overdue_amount: amount,
                    ..
                } => Some(CoreCreditCollectionsEvent::ObligationOverdue {
                    id: entity.id,
                    credit_facility_id: entity.credit_facility_id,
                    amount: *amount,
                }),
                DefaultedRecorded {
                    defaulted_amount: amount,
                    ..
                } => Some(CoreCreditCollectionsEvent::ObligationDefaulted {
                    id: entity.id,
                    credit_facility_id: entity.credit_facility_id,
                    amount: *amount,
                }),
                Completed { .. } => Some(CoreCreditCollectionsEvent::ObligationCompleted {
                    id: entity.id,
                    credit_facility_id: entity.credit_facility_id,
                }),
                _ => None,
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }
}
