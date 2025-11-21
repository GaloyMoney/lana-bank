use outbox::{Outbox, OutboxEventMarker};

use crate::{event::CorePaymentLinkEvent, funding_link::error::FundingLinkError};

pub(super) struct PaymentLinkPublisher<E>
where
    E: OutboxEventMarker<CorePaymentLinkEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for PaymentLinkPublisher<E>
where
    E: OutboxEventMarker<CorePaymentLinkEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> PaymentLinkPublisher<E>
where
    E: OutboxEventMarker<CorePaymentLinkEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    pub async fn publish(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        event: CorePaymentLinkEvent,
    ) -> Result<(), sqlx::Error> {
        self.outbox.publish_persisted(op, event).await?;
        Ok(())
    }

    pub async fn publish_all(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        events: impl Iterator<Item = CorePaymentLinkEvent>,
    ) -> Result<(), FundingLinkError> {
        self.outbox.publish_all_persisted(op, events).await?;
        Ok(())
    }
}

