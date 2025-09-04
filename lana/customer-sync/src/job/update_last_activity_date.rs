use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_deposit::{CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject};
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use outbox::{EventSequence, Outbox, OutboxEventMarker};

use lana_events::LanaEvent;

#[derive(Default, Clone, Deserialize, Serialize)]
struct UpdateLastActivityDateJobData {
    sequence: EventSequence,
}

pub struct UpdateLastActivityDateJobRunner<Perms, E>
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
    outbox: Outbox<E>,
    deposits: CoreDeposit<Perms, E>,
    customers: Customers<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for UpdateLastActivityDateJobRunner<Perms, E>
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
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<UpdateLastActivityDateJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            if let Some(event) = &message.payload {
                let event = if let Some(event) = event.as_event() {
                    event
                } else {
                    continue;
                };
                // TODO: Add other events that should update the customer activity
                let customer_id = match event {
                    LanaEvent::Deposit(
                        CoreDepositEvent::DepositInitialized {
                            deposit_account_id, ..
                        }
                        | CoreDepositEvent::WithdrawalConfirmed {
                            deposit_account_id, ..
                        }
                        | CoreDepositEvent::DepositReverted {
                            deposit_account_id, ..
                        },
                    ) => {
                        let account = self
                            .deposits
                            .find_account_by_id_without_audit(*deposit_account_id)
                            .await?;
                        Some(account.account_holder_id.into())
                    }
                    _ => None,
                };

                if let Some(customer_id) = customer_id {
                    let activity_date = message.recorded_at;
                    self.customers
                        .record_last_activity_date(customer_id, activity_date)
                        .await?;
                }
            }

            state.sequence = message.sequence;
            current_job.update_execution_state(&state).await?;
        }

        Ok(JobCompletion::RescheduleNow)
    }
}

impl<Perms, E> UpdateLastActivityDateJobRunner<Perms, E>
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
    pub fn new(
        outbox: &Outbox<E>,
        customers: &Customers<Perms, E>,
        deposits: &CoreDeposit<Perms, E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            customers: customers.clone(),
            deposits: deposits.clone(),
        }
    }
}

pub struct UpdateLastActivityDateInit<Perms, E>
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
    outbox: Outbox<E>,
    customers: Customers<Perms, E>,
    deposits: CoreDeposit<Perms, E>,
}

impl<Perms, E> UpdateLastActivityDateInit<Perms, E>
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
    pub fn new(
        outbox: &Outbox<E>,
        customers: &Customers<Perms, E>,
        deposits: &CoreDeposit<Perms, E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            customers: customers.clone(),
            deposits: deposits.clone(),
        }
    }
}

const UPDATE_LAST_ACTIVITY_DATE: JobType = JobType::new("update-last-activity-date");

impl<Perms, E> JobInitializer for UpdateLastActivityDateInit<Perms, E>
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
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        UPDATE_LAST_ACTIVITY_DATE
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UpdateLastActivityDateJobRunner::new(
            &self.outbox,
            &self.customers,
            &self.deposits,
        )))
    }

    fn retry_on_error_settings() -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateLastActivityDateConfig<Perms, E> {
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> UpdateLastActivityDateConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> Default for UpdateLastActivityDateConfig<Perms, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Perms, E> JobConfig for UpdateLastActivityDateConfig<Perms, E>
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
    type Initializer = UpdateLastActivityDateInit<Perms, E>;
}
