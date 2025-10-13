use async_trait::async_trait;
use futures::StreamExt;
use tracing::{Span, instrument};

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, KycVerification};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, DepositAccountStatus,
    GovernanceAction, GovernanceObject,
};
use governance::GovernanceEvent;
use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use job::*;

use crate::config::*;

#[derive(serde::Serialize)]
pub struct CustomerActiveSyncJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> CustomerActiveSyncJobConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<Perms, E> JobConfig for CustomerActiveSyncJobConfig<Perms, E>
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
    type Initializer = CustomerActiveSyncInit<Perms, E>;
}

pub struct CustomerActiveSyncInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    outbox: Outbox<E>,
    deposit: CoreDeposit<Perms, E>,
    config: CustomerSyncConfig,
}

impl<Perms, E> CustomerActiveSyncInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        deposit: &CoreDeposit<Perms, E>,
        config: CustomerSyncConfig,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            deposit: deposit.clone(),
            config,
        }
    }
}

const CUSTOMER_ACTIVE_SYNC: JobType = JobType::new("customer-active-sync");
impl<Perms, E> JobInitializer for CustomerActiveSyncInit<Perms, E>
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
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CUSTOMER_ACTIVE_SYNC
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CustomerActiveSyncJobRunner {
            outbox: self.outbox.clone(),
            deposit: self.deposit.clone(),
            config: self.config.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
struct CustomerActiveSyncJobData {
    sequence: outbox::EventSequence,
}

pub struct CustomerActiveSyncJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    outbox: Outbox<E>,
    deposit: CoreDeposit<Perms, E>,
    config: CustomerSyncConfig,
}

impl<Perms, E> CustomerActiveSyncJobRunner<Perms, E>
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
    #[instrument(name = "customer_sync.active_sync_job.process_msg", parent = None, skip(self, message), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    #[allow(clippy::single_match)]
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match message.as_event() {
            Some(
                event @ CoreCustomerEvent::CustomerAccountKycVerificationUpdated {
                    id,
                    kyc_verification,
                    ..
                },
            ) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());
                self.handle_status_updated(*id, *kyc_verification).await?;
            }
            _ => {}
        }
        Ok(())
    }

    #[instrument(name = "customer_sync.active_sync_job.handle", skip(self), fields(id = ?id, kyc = ?kyc_verification))]
    async fn handle_status_updated(
        &self,
        id: core_customer::CustomerId,
        kyc_verification: KycVerification,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.customer_status_sync_active {
            let deposit_account_status = match kyc_verification {
                KycVerification::Rejected | KycVerification::PendingVerification => {
                    DepositAccountStatus::Inactive
                }
                KycVerification::Verified => DepositAccountStatus::Active,
            };

            self.deposit
                .update_account_status_for_holder(
                    &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(),
                    id,
                    deposit_account_status,
                )
                .await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<Perms, E> JobRunner for CustomerActiveSyncJobRunner<Perms, E>
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
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CustomerActiveSyncJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            self.process_message(message.as_ref()).await?;
            state.sequence = message.sequence;
            current_job.update_execution_state(&state).await?;
        }

        Ok(JobCompletion::RescheduleNow)
    }
}
