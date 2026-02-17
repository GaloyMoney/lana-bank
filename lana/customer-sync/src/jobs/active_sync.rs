use tracing::{Span, instrument};

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, KycVerification};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject,
    DepositAccountHolderStatus, GovernanceAction, GovernanceObject,
};
use governance::GovernanceEvent;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::JobType;

pub const CUSTOMER_ACTIVE_SYNC: JobType = JobType::new("outbox.customer-active-sync");

pub struct CustomerActiveSyncHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    deposit: CoreDeposit<Perms, E>,
}

impl<Perms, E> CustomerActiveSyncHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(deposit: &CoreDeposit<Perms, E>) -> Self {
        Self {
            deposit: deposit.clone(),
        }
    }
}

impl<Perms, E> OutboxEventHandler<E> for CustomerActiveSyncHandler<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    #[instrument(name = "customer_sync.active_sync_job.process_message", parent = None, skip(self, _op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        _op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreCustomerEvent::CustomerKycUpdated { entity }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());
            self.handle_status_updated(entity.id, entity.kyc_verification)
                .await?;
        }
        Ok(())
    }
}

impl<Perms, E> CustomerActiveSyncHandler<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    #[instrument(name = "customer_sync.active_sync_job.handle", skip(self), fields(id = ?id, kyc = ?kyc_verification))]
    async fn handle_status_updated(
        &self,
        id: core_customer::CustomerId,
        kyc_verification: KycVerification,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let deposit_account_status = match kyc_verification {
            KycVerification::Rejected | KycVerification::PendingVerification => {
                DepositAccountHolderStatus::Inactive
            }
            KycVerification::Verified => DepositAccountHolderStatus::Active,
        };

        self.deposit
            .update_account_status_for_holder(
                &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(
                    core_customer::CUSTOMER_SYNC,
                ),
                id,
                deposit_account_status,
            )
            .await?;
        Ok(())
    }
}
