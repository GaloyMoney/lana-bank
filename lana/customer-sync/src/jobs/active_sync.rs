use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, KycVerification};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject,
    DepositAccountHolderStatus, GovernanceAction, GovernanceObject,
};
use governance::GovernanceEvent;
use obix::out::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use job::*;

#[derive(Serialize, Deserialize)]
pub(crate) struct CustomerActiveSyncJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> CustomerActiveSyncJobConfig<Perms, E> {
    pub(crate) fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

pub(crate) struct CustomerActiveSyncInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    outbox: Outbox<E>,
    deposit: CoreDeposit<Perms, E>,
}

impl<Perms, E> CustomerActiveSyncInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub(crate) fn new(outbox: &Outbox<E>, deposit: &CoreDeposit<Perms, E>) -> Self {
        Self {
            outbox: outbox.clone(),
            deposit: deposit.clone(),
        }
    }
}

const CUSTOMER_ACTIVE_SYNC: JobType = JobType::new("outbox.customer-active-sync");
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
    type Config = CustomerActiveSyncJobConfig<Perms, E>;
    fn job_type(&self) -> JobType {
        CUSTOMER_ACTIVE_SYNC
    }

    fn init(
        &self,
        _: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CustomerActiveSyncJobRunner {
            outbox: self.outbox.clone(),
            deposit: self.deposit.clone(),
        }))
    }

    fn retry_on_error_settings(&self) -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
struct CustomerActiveSyncJobData {
    sequence: obix::EventSequence,
}

pub(crate) struct CustomerActiveSyncJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    outbox: Outbox<E>,
    deposit: CoreDeposit<Perms, E>,
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
    #[instrument(name = "customer_sync.active_sync_job.process_message", parent = None, skip(self, message), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    #[allow(clippy::single_match)]
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match message.as_event() {
            Some(event @ CoreCustomerEvent::CustomerKycUpdated { entity }) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());
                self.handle_status_updated(entity.id, entity.kyc_verification)
                    .await?;
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
        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %CUSTOMER_ACTIVE_SYNC,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(message) => {
                            self.process_message(message.as_ref()).await?;
                            state.sequence = message.sequence;
                            current_job.update_execution_state(&state).await?;
                        }
                        None => {
                            return Ok(JobCompletion::RescheduleNow);
                        }
                    }
                }
            }
        }
    }
}
