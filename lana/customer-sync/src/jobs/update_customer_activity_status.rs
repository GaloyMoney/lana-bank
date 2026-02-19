use tracing::{Span, instrument};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_deposit::{
    CoreDepositAction, CoreDepositEvent, CoreDepositObject, GovernanceAction, GovernanceObject,
};
use core_time_events::CoreTimeEvent;
use governance::GovernanceEvent;
use lana_events::LanaEvent;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::JobType;

pub const UPDATE_CUSTOMER_ACTIVITY_STATUS: JobType =
    JobType::new("outbox.update-customer-activity-status");

pub struct UpdateCustomerActivityStatusHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    customers: Customers<Perms, E>,
}

impl<Perms, E> UpdateCustomerActivityStatusHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    pub fn new(customers: &Customers<Perms, E>) -> Self {
        Self {
            customers: customers.clone(),
        }
    }
}

impl<Perms, E> OutboxEventHandler<E> for UpdateCustomerActivityStatusHandler<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    #[instrument(name = "customer_sync.update_customer_activity_status.process_message", parent = None, skip(self, _op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        _op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreTimeEvent::EndOfDay { closing_time, .. }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.customers
                .perform_customer_activity_status_update(*closing_time)
                .await?;
        }
        Ok(())
    }
}
