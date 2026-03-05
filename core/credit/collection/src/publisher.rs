use obix::out::{Outbox, OutboxEventMarker};
use tracing::instrument;

use crate::{
    CoreCreditCollectionEvent, PublicObligation, PublicPayment, PublicPaymentAllocation,
    obligation::{Obligation, ObligationEvent},
    payment::{Payment, PaymentEvent},
    payment_allocation::{PaymentAllocation, PaymentAllocationEvent},
};

pub struct CollectionPublisher<E>
where
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for CollectionPublisher<E>
where
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> CollectionPublisher<E>
where
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    #[instrument(name = "collection.publisher.publish_payment_in_op", skip_all)]
    pub async fn publish_payment_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Payment,
        new_events: es_entity::LastPersisted<'_, PaymentEvent>,
    ) -> Result<(), sqlx::Error> {
        use PaymentEvent::*;
        let publish_events = new_events
            .map(|event| match &event.event {
                Initialized { .. } => CoreCreditCollectionEvent::PaymentCreated {
                    entity: PublicPayment::from(entity),
                },
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }

    #[instrument(
        name = "collection.publisher.publish_payment_allocation_in_op",
        skip_all
    )]
    pub async fn publish_payment_allocation_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &PaymentAllocation,
        new_events: es_entity::LastPersisted<'_, PaymentAllocationEvent>,
    ) -> Result<(), sqlx::Error> {
        use PaymentAllocationEvent::*;
        let publish_events = new_events
            .map(|event| match &event.event {
                Initialized { .. } => CoreCreditCollectionEvent::PaymentAllocationCreated {
                    entity: PublicPaymentAllocation::from(entity),
                },
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }

    #[instrument(name = "collection.publisher.publish_obligation_in_op", skip_all)]
    pub async fn publish_obligation_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Obligation,
        new_events: es_entity::LastPersisted<'_, ObligationEvent>,
    ) -> Result<(), sqlx::Error> {
        use ObligationEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Initialized { .. } => Some(CoreCreditCollectionEvent::ObligationCreated {
                    entity: PublicObligation::from(entity),
                }),
                DueRecorded { .. } => Some(CoreCreditCollectionEvent::ObligationDue {
                    entity: PublicObligation::from(entity),
                }),
                OverdueRecorded { .. } => Some(CoreCreditCollectionEvent::ObligationOverdue {
                    entity: PublicObligation::from(entity),
                }),
                DefaultedRecorded { .. } => Some(CoreCreditCollectionEvent::ObligationDefaulted {
                    entity: PublicObligation::from(entity),
                }),
                Completed { .. } => Some(CoreCreditCollectionEvent::ObligationCompleted {
                    entity: PublicObligation::from(entity),
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
