use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CustomerObject};
use core_deposit::{CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject};
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use outbox::{EventSequence, Outbox, OutboxEventMarker};

use crate::CustomerActivityRepo;
use lana_events::LanaEvent;

#[derive(Default, Clone, Deserialize, Serialize)]
struct CustomerActivityUpdateJobData {
    sequence: EventSequence,
}

pub struct CustomerActivityUpdateJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    outbox: Outbox<E>,
    repo: CustomerActivityRepo,
    deposits: CoreDeposit<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for CustomerActivityUpdateJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CustomerActivityUpdateJobData>()?
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
                    self.repo
                        .upsert_activity(customer_id, activity_date)
                        .await?;
                }
            }

            state.sequence = message.sequence;
            current_job.update_execution_state(&state).await?;
        }

        Ok(JobCompletion::RescheduleNow)
    }
}

impl<Perms, E> CustomerActivityUpdateJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        repo: &CustomerActivityRepo,
        deposits: &CoreDeposit<Perms, E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            repo: repo.clone(),
            deposits: deposits.clone(),
        }
    }
}

pub struct CustomerActivityUpdateInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    outbox: Outbox<E>,
    repo: CustomerActivityRepo,
    deposits: CoreDeposit<Perms, E>,
}

impl<Perms, E> CustomerActivityUpdateInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(outbox: &Outbox<E>, pool: sqlx::PgPool, deposits: &CoreDeposit<Perms, E>) -> Self {
        let repo = CustomerActivityRepo::new(pool);
        Self {
            outbox: outbox.clone(),
            repo,
            deposits: deposits.clone(),
        }
    }
}

const CUSTOMER_ACTIVITY_UPDATE: JobType = JobType::new("customer-activity-update");

impl<Perms, E> JobInitializer for CustomerActivityUpdateInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CUSTOMER_ACTIVITY_UPDATE
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CustomerActivityUpdateJobRunner::new(
            &self.outbox,
            &self.repo,
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
pub struct CustomerActivityUpdateConfig<Perms, E> {
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> CustomerActivityUpdateConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> Default for CustomerActivityUpdateConfig<Perms, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Perms, E> JobConfig for CustomerActivityUpdateConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    type Initializer = CustomerActivityUpdateInit<Perms, E>;
}
