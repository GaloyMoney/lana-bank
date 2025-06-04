use outbox::{Outbox, OutboxEventMarker};

use crate::event::*;

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
