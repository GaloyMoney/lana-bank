use tracing::{Span, instrument};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, GovernanceAction,
    GovernanceObject,
};
use governance::GovernanceEvent;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::JobType;
use lana_events::LanaEvent;

pub const UPDATE_DEPOSIT_ACCOUNT_ACTIVITY_DATE: JobType =
    JobType::new("outbox.update-deposit-account-activity-date");

pub struct UpdateDepositAccountActivityDateHandler<Perms, E>
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
}

impl<Perms, E> UpdateDepositAccountActivityDateHandler<Perms, E>
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
    pub fn new(deposits: &CoreDeposit<Perms, E>) -> Self {
        Self {
            deposits: deposits.clone(),
        }
    }
}

impl<Perms, E> OutboxEventHandler<E> for UpdateDepositAccountActivityDateHandler<Perms, E>
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
    #[instrument(name = "deposit_sync.update_deposit_account_activity_date.process_message", parent = None, skip(self, _op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
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

        let activity_date = event.recorded_at;

        self.deposits
            .record_deposit_account_activity(deposit_account_id, activity_date)
            .await?;
        Ok(())
    }
}
