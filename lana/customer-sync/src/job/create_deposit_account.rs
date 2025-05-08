use async_trait::async_trait;
use futures::StreamExt;
use tracing::instrument;

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject};
use deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, GovernanceAction,
    GovernanceObject,
};
use governance::GovernanceEvent;
use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use job::*;

use crate::config::*;

#[derive(serde::Serialize)]
pub struct CreateDepositAccountJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> CreateDepositAccountJobConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<Perms, E> JobConfig for CreateDepositAccountJobConfig<Perms, E>
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
    type Initializer = CreateDepositAccountJobInitializer<Perms, E>;
}

pub struct CreateDepositAccountJobInitializer<Perms, E>
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

impl<Perms, E> CreateDepositAccountJobInitializer<Perms, E>
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

const CUSTOMER_SYNC_CREATE_DEPOSIT_ACCOUNT: JobType =
    JobType::new("customer-sync-create-deposit-account");
impl<Perms, E> JobInitializer for CreateDepositAccountJobInitializer<Perms, E>
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
        CUSTOMER_SYNC_CREATE_DEPOSIT_ACCOUNT
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreateDepositAccountJobRunner {
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
struct CreateDepositAccountJobData {
    sequence: outbox::EventSequence,
}

pub struct CreateDepositAccountJobRunner<Perms, E>
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
#[async_trait]
impl<Perms, E> JobRunner for CreateDepositAccountJobRunner<Perms, E>
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
            .execution_state::<CreateDepositAccountJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            let did_handle = match message.as_ref().as_event() {
                Some(CoreCustomerEvent::CustomerCreated {
                    id, customer_type, ..
                }) if self.config.create_deposit_account_on_customer_create => {
                    self.handle_create_deposit_account(message.as_ref(), *id, *customer_type, true)
                        .await?;
                    true
                }
                Some(CoreCustomerEvent::CustomerAccountStatusUpdated {
                    id,
                    customer_type,
                    status: core_customer::AccountStatus::Active,
                    ..
                }) if !self.config.create_deposit_account_on_customer_create => {
                    self.handle_create_deposit_account(
                        message.as_ref(),
                        *id,
                        *customer_type,
                        false,
                    )
                    .await?;
                    true
                }
                _ => false,
            };

            if did_handle {
                state.sequence = message.sequence;
                current_job.update_execution_state(&state).await?;
            }
        }

        Ok(JobCompletion::RescheduleNow)
    }
}

impl<Perms, E> CreateDepositAccountJobRunner<Perms, E>
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
    #[instrument(name = "customer_sync.create_deposit_account", skip(self, message))]
    async fn handle_create_deposit_account(
        &self,
        message: &PersistentOutboxEvent<E>,
        id: core_customer::CustomerId,
        customer_type: core_customer::CustomerType,
        is_customer_create_event: bool,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        E: OutboxEventMarker<CoreCustomerEvent>,
    {
        message.inject_trace_parent();

        // don't activate if we are syncing the customer status
        let active = !(is_customer_create_event && self.config.customer_status_sync_active);

        if self.config.auto_create_deposit_account {
            match self.deposit
                .create_account(
                    &<<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject as SystemSubject>::system(),
                    id,
                    active,
                    customer_type,
                )
                .await
            {
                Ok(_) => {}
                Err(e) if e.is_account_already_exists() => {},
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }
}
