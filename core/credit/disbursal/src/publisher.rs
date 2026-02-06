use obix::out::{Outbox, OutboxEventMarker};
use tracing::instrument;
use tracing_macros::record_error_severity;

use crate::{
    entity::{Disbursal, DisbursalEvent},
    error::DisbursalError,
    event::CoreCreditDisbursalEvent,
};

pub struct DisbursalPublisher<E>
where
    E: OutboxEventMarker<CoreCreditDisbursalEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for DisbursalPublisher<E>
where
    E: OutboxEventMarker<CoreCreditDisbursalEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> DisbursalPublisher<E>
where
    E: OutboxEventMarker<CoreCreditDisbursalEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    #[record_error_severity]
    #[instrument(name = "disbursal.publisher.publish_disbursal_in_op", skip_all)]
    pub async fn publish_disbursal_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Disbursal,
        new_events: es_entity::LastPersisted<'_, DisbursalEvent>,
    ) -> Result<(), DisbursalError> {
        use DisbursalEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Settled {
                    amount,
                    ledger_tx_id,
                    effective,
                    ..
                } => Some(CoreCreditDisbursalEvent::DisbursalSettled {
                    beneficiary_id: entity.beneficiary_id,
                    amount: *amount,
                    recorded_at: event.recorded_at,
                    effective: *effective,
                    ledger_tx_id: *ledger_tx_id,
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
