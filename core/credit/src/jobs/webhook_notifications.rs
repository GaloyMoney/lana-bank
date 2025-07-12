use async_trait::async_trait;
use futures::StreamExt;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject};
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use outbox::{Outbox, OutboxEventMarker};

use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};

use crate::{CoreCredit, CoreCreditAction, CoreCreditEvent, CoreCreditObject};

#[derive(serde::Serialize)]
pub struct WebhookNotificationsJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> WebhookNotificationsJobConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> Default for WebhookNotificationsJobConfig<Perms, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Perms, E> JobConfig for WebhookNotificationsJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    type Initializer = WebhookNotificationsInit<Perms, E>;
}

#[derive(Default, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct WebhookNotificationsJobData {
    sequence: outbox::EventSequence,
}

pub struct WebhookNotificationsJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    credit: CoreCredit<Perms, E>,
    outbox: Outbox<E>,
}

#[async_trait]
impl<Perms, E> JobRunner for WebhookNotificationsJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<WebhookNotificationsJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            match message.as_ref().as_event() {
                Some(CoreCustodyEvent::WalletBalanceChanged {
                    external_wallet_id,
                    amount,
                }) => {
                    self.credit
                        .update_collateral_by_custodian(external_wallet_id, *amount)
                        .await?;
                    state.sequence = message.sequence;
                    current_job.update_execution_state(state).await?;
                }
                _ => {}
            }
        }

        Ok(JobCompletion::RescheduleNow)
    }
}

pub struct WebhookNotificationsInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    outbox: Outbox<E>,
    credit: CoreCredit<Perms, E>,
}

impl<Perms, E> WebhookNotificationsInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    pub fn new(outbox: &Outbox<E>, credit: &CoreCredit<Perms, E>) -> Self {
        Self {
            outbox: outbox.clone(),
            credit: credit.clone(),
        }
    }
}

const WEBHOOK_NOTIFICATIONS_JOB: JobType = JobType::new("webhook-notifications");
impl<Perms, E> JobInitializer for WebhookNotificationsInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        WEBHOOK_NOTIFICATIONS_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(WebhookNotificationsJobRunner {
            outbox: self.outbox.clone(),
            credit: self.credit.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}
