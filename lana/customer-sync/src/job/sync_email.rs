use async_trait::async_trait;
use futures::StreamExt;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject};
use keycloak_client::KeycloakClient;
use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use job::*;

#[derive(serde::Serialize)]
pub struct SyncEmailJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> SyncEmailJobConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> JobConfig for SyncEmailJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CustomerObject>,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    type Initializer = SyncEmailInit<Perms, E>;
}

pub struct SyncEmailInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    outbox: Outbox<E>,
    keycloak_client: KeycloakClient,
    _phantom: std::marker::PhantomData<Perms>,
}

impl<Perms, E> SyncEmailInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    pub fn new(outbox: &Outbox<E>, keycloak_client: KeycloakClient) -> Self {
        Self {
            outbox: outbox.clone(),
            keycloak_client,
            _phantom: std::marker::PhantomData,
        }
    }
}

const SYNC_EMAIL_JOB: JobType = JobType::new("sync-email-job");
impl<Perms, E> JobInitializer for SyncEmailInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CustomerObject>,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        SYNC_EMAIL_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(SyncEmailJobRunner::<Perms, E> {
            outbox: self.outbox.clone(),
            keycloak_client: self.keycloak_client.clone(),
            _phantom: std::marker::PhantomData,
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
struct SyncEmailJobData {
    sequence: outbox::EventSequence,
}

pub struct SyncEmailJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    outbox: Outbox<E>,
    keycloak_client: KeycloakClient,
    _phantom: std::marker::PhantomData<Perms>,
}

#[async_trait]
impl<Perms, E> JobRunner for SyncEmailJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CustomerObject>,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<SyncEmailJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            if let Some(CoreCustomerEvent::CustomerEmailUpdated { .. }) =
                &message.as_ref().as_event()
            {
                self.handle_email_update(message.as_ref()).await?;
                state.sequence = message.sequence;
                current_job.update_execution_state(&state).await?;
            }
        }

        Ok(JobCompletion::RescheduleNow)
    }
}

impl<Perms, E> SyncEmailJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CustomerObject>,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    #[instrument(name = "customer_sync.sync_email", skip(self, message))]
    async fn handle_email_update(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        E: OutboxEventMarker<CoreCustomerEvent>,
    {
        if let Some(CoreCustomerEvent::CustomerEmailUpdated { id, email }) = message.as_event() {
            message.inject_trace_parent();

            // We no longer need to read from Customers; use the id directly
            self.keycloak_client
                .update_user_email((*id).into(), email.clone())
                .await?;
        }
        Ok(())
    }
}
