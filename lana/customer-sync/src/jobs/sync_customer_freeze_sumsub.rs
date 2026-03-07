use tracing::{Span, instrument};

use job::{JobId, JobSpawner, JobType};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use core_customer::{CoreCustomerEvent, CustomerId};
#[cfg(test)]
use core_customer::{CustomerStatus, PartyId, PublicCustomer};
#[cfg(test)]
use uuid::Uuid;

use super::deactivate_sumsub_applicant::DeactivateSumsubApplicantConfig;

pub const CUSTOMER_SYNC_DEACTIVATE_SUMSUB_APPLICANT: JobType =
    JobType::new("outbox.customer-sync-deactivate-sumsub-applicant");

pub struct SyncCustomerFreezeSumsubHandler {
    deactivate_sumsub_applicant: JobSpawner<DeactivateSumsubApplicantConfig>,
}

impl SyncCustomerFreezeSumsubHandler {
    pub fn new(deactivate_sumsub_applicant: JobSpawner<DeactivateSumsubApplicantConfig>) -> Self {
        Self {
            deactivate_sumsub_applicant,
        }
    }
}

fn customer_id_to_deactivate(event: &CoreCustomerEvent) -> Option<CustomerId> {
    match event {
        CoreCustomerEvent::CustomerFrozen { entity }
        | CoreCustomerEvent::CustomerClosed { entity } => Some(entity.id),
        _ => None,
    }
}

impl<E> OutboxEventHandler<E> for SyncCustomerFreezeSumsubHandler
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    #[instrument(name = "customer_sync.deactivate_sumsub_applicant_sync_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let Some(core_event) = event.as_event() else {
            return Ok(());
        };
        let Some(customer_id) = customer_id_to_deactivate(core_event) else {
            return Ok(());
        };

        event.inject_trace_parent();
        Span::current().record("handled", true);
        Span::current().record("event_type", core_event.as_ref());

        self.deactivate_sumsub_applicant
            .spawn_with_queue_id_in_op(
                op,
                JobId::new(),
                DeactivateSumsubApplicantConfig { customer_id },
                customer_id.to_string(),
            )
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
    fn deactivate_sumsub_on_customer_frozen() {
        let event = CoreCustomerEvent::CustomerFrozen {
            entity: public_customer(CustomerStatus::Frozen),
        };

        assert_eq!(
            customer_id_to_deactivate(&event),
            Some(match event {
                CoreCustomerEvent::CustomerFrozen { entity } => entity.id,
                _ => unreachable!(),
            })
        );
    }

    #[test]
    fn deactivate_sumsub_on_customer_closed() {
        let event = CoreCustomerEvent::CustomerClosed {
            entity: public_customer(CustomerStatus::Closed),
        };

        assert_eq!(
            customer_id_to_deactivate(&event),
            Some(match event {
                CoreCustomerEvent::CustomerClosed { entity } => entity.id,
                _ => unreachable!(),
            })
        );
    }

    #[test]
    fn ignore_non_freeze_events() {
        let active = CoreCustomerEvent::CustomerCreated {
            entity: public_customer(CustomerStatus::Active),
        };
        let unfrozen = CoreCustomerEvent::CustomerUnfrozen {
            entity: public_customer(CustomerStatus::Active),
        };

        assert_eq!(customer_id_to_deactivate(&active), None);
        assert_eq!(customer_id_to_deactivate(&unfrozen), None);
    }
}
