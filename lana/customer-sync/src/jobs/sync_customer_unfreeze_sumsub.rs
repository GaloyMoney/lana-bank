use tracing::{Span, instrument};

use job::{JobId, JobSpawner, JobType};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use core_customer::{CoreCustomerEvent, CustomerId};
#[cfg(test)]
use core_customer::{CustomerStatus, PartyId, PublicCustomer};

use super::activate_sumsub_applicant::ActivateSumsubApplicantConfig;

pub const CUSTOMER_SYNC_ACTIVATE_SUMSUB_APPLICANT: JobType =
    JobType::new("outbox.customer-sync-activate-sumsub-applicant");

pub struct SyncCustomerUnfreezeSumsubHandler {
    activate_sumsub_applicant: JobSpawner<ActivateSumsubApplicantConfig>,
}

impl SyncCustomerUnfreezeSumsubHandler {
    pub fn new(activate_sumsub_applicant: JobSpawner<ActivateSumsubApplicantConfig>) -> Self {
        Self {
            activate_sumsub_applicant,
        }
    }
}

fn customer_id_to_activate(event: &CoreCustomerEvent) -> Option<CustomerId> {
    match event {
        CoreCustomerEvent::CustomerUnfrozen { entity } => Some(entity.id),
        _ => None,
    }
}

impl<E> OutboxEventHandler<E> for SyncCustomerUnfreezeSumsubHandler
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    #[instrument(name = "customer_sync.activate_sumsub_applicant_sync_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let Some(core_event) = event.as_event() else {
            return Ok(());
        };
        let Some(customer_id) = customer_id_to_activate(core_event) else {
            return Ok(());
        };

        event.inject_trace_parent();
        Span::current().record("handled", true);
        Span::current().record("event_type", core_event.as_ref());

        self.activate_sumsub_applicant
            .spawn_with_queue_id_in_op(
                op,
                JobId::new(),
                ActivateSumsubApplicantConfig { customer_id },
                customer_id.to_string(),
            )
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    fn public_customer(status: CustomerStatus) -> PublicCustomer {
        let customer_id = CustomerId::from(Uuid::new_v4());
        PublicCustomer {
            id: customer_id,
            party_id: PartyId::from(customer_id),
            status,
        }
    }

    #[test]
    fn activate_sumsub_on_customer_unfrozen() {
        let event = CoreCustomerEvent::CustomerUnfrozen {
            entity: public_customer(CustomerStatus::Active),
        };

        assert_eq!(
            customer_id_to_activate(&event),
            Some(match event {
                CoreCustomerEvent::CustomerUnfrozen { entity } => entity.id,
                _ => unreachable!(),
            })
        );
    }

    #[test]
    fn ignore_non_unfreeze_events() {
        let frozen = CoreCustomerEvent::CustomerFrozen {
            entity: public_customer(CustomerStatus::Frozen),
        };
        let closed = CoreCustomerEvent::CustomerClosed {
            entity: public_customer(CustomerStatus::Closed),
        };

        assert_eq!(customer_id_to_activate(&frozen), None);
        assert_eq!(customer_id_to_activate(&closed), None);
    }
}
