use obix::out::{Outbox, OutboxEventMarker};

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
        op: &mut impl es_entity::AtomicOperation,
        entity: &Wallet,
        new_events: es_entity::LastPersisted<'_, WalletEvent>,
    ) -> Result<(), WalletError> {
        use WalletEvent::*;
        let events = new_events
            .filter_map(|event| match &event.event {
                Initialized { .. } => None,

                BalanceChanged {
                    new_balance,
                    changed_at,
                    ..
                } => Some(CoreCustodyEvent::WalletBalanceChanged {
                    id: entity.id,
                    new_balance: *new_balance,
                    changed_at: *changed_at,
                }),
            })
            .collect::<Vec<_>>();

        self.outbox.publish_all_persisted(op, events).await?;

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
