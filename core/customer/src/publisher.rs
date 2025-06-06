use outbox::{Outbox, OutboxEventMarker};

use super::{entity::*, error::*, event::*};

pub struct CustomerPublisher<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for CustomerPublisher<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> CustomerPublisher<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    pub async fn publish(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &Customer,
        new_events: es_entity::LastPersisted<'_, CustomerEvent>,
    ) -> Result<(), CustomerError> {
        use CustomerEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Initialized { .. } => Some(CoreCustomerEvent::CustomerCreated {
                    id: entity.id,
                    email: entity.email.clone(),
                    customer_type: entity.customer_type,
                }),
                AccountStatusUpdated { status, .. } => {
                    Some(CoreCustomerEvent::CustomerAccountStatusUpdated {
                        id: entity.id,
                        status: *status,
                        customer_type: entity.customer_type,
                    })
                }
                EmailUpdated { email, .. } => Some(CoreCustomerEvent::CustomerEmailUpdated {
                    id: entity.id,
                    email: email.clone(),
                }),
                _ => None,
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(db.tx(), publish_events)
            .await?;
        Ok(())
    }
}
