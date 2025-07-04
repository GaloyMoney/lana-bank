use outbox::{Outbox, OutboxEventMarker};

use crate::{
    CoreCustodyEvent,
    wallet::{Wallet, WalletEvent, error::WalletError},
};

pub struct CustodyPublisher<E>
where
    E: OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
}

impl<E> CustodyPublisher<E>
where
    E: OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    pub async fn publish_wallet(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &Wallet,
        new_events: es_entity::LastPersisted<'_, WalletEvent>,
    ) -> Result<(), WalletError> {
        use WalletEvent::*;
        let events = new_events
            .filter_map(|event| match &event.event {
                Initialized { .. } => None,
                ExternalWalletAttached { address, .. } => Some(CoreCustodyEvent::WalletAttached {
                    id: entity.id,
                    address: address.to_owned(),
                }),
            })
            .collect::<Vec<_>>();

        self.outbox.publish_all_persisted(db.tx(), events).await?;

        Ok(())
    }
}

impl<E> Clone for CustodyPublisher<E>
where
    E: OutboxEventMarker<CoreCustodyEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}
