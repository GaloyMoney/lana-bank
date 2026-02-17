use tracing::{Span, instrument};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, GovernanceAction,
    GovernanceObject,
};
use governance::GovernanceEvent;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::JobType;
use lana_events::LanaEvent;

pub const UPDATE_LAST_ACTIVITY_DATE: JobType = JobType::new("outbox.update-last-activity-date");

pub struct UpdateLastActivityDateHandler<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    deposits: CoreDeposit<Perms, E>,
    customers: Customers<Perms, E>,
}

impl<Perms, E> UpdateLastActivityDateHandler<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    pub fn new(customers: &Customers<Perms, E>, deposits: &CoreDeposit<Perms, E>) -> Self {
        Self {
            customers: customers.clone(),
            deposits: deposits.clone(),
        }
    }
}

impl<Perms, E> OutboxEventHandler<E> for UpdateLastActivityDateHandler<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    #[instrument(name = "customer_sync.update_last_activity_date_job.process_message", parent = None, skip(self, _op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        _op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (e, deposit_account_id) = match event.as_event() {
            Some(e @ CoreDepositEvent::DepositInitialized { entity }) => {
                (e, entity.deposit_account_id)
            }
            Some(e @ CoreDepositEvent::WithdrawalConfirmed { entity }) => {
                (e, entity.deposit_account_id)
            }
            Some(e @ CoreDepositEvent::DepositReverted { entity }) => {
                (e, entity.deposit_account_id)
            }
            _ => return Ok(()),
        };

        event.inject_trace_parent();
        Span::current().record("handled", true);
        Span::current().record("event_type", e.as_ref());

        let account = self
            .deposits
            .find_account_by_id_without_audit(deposit_account_id)
            .await?;

        let customer_id = account.account_holder_id.into();
        let activity_date = event.recorded_at;

        self.customers
            .record_last_activity_date(customer_id, activity_date)
            .await?;
        Ok(())
    }
}
