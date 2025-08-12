use async_trait::async_trait;
use futures::StreamExt;
use tracing::instrument;

use core_customer::CoreCustomerEvent;
use keycloak_client::KeycloakClient;
use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use job::*;

#[derive(serde::Serialize)]
pub struct SyncEmailJobConfig<E> {
    _phantom: std::marker::PhantomData<E>,
}

impl<E> SyncEmailJobConfig<E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E> JobConfig for SyncEmailJobConfig<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    type Initializer = SyncEmailInit<E>;
}

pub struct SyncEmailInit<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    outbox: Outbox<E>,
    keycloak_client: KeycloakClient,
}

impl<E> SyncEmailInit<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    pub fn new(outbox: &Outbox<E>, keycloak_client: KeycloakClient) -> Self {
        Self {
            outbox: outbox.clone(),
            keycloak_client,
        }
    }
}

const SYNC_EMAIL_JOB: JobType = JobType::new("sync-email-job");
impl<E> JobInitializer for SyncEmailInit<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        SYNC_EMAIL_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(SyncEmailJobRunner::<E> {
            outbox: self.outbox.clone(),
            keycloak_client: self.keycloak_client.clone(),
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

pub struct SyncEmailJobRunner<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    outbox: Outbox<E>,
    keycloak_client: KeycloakClient,
}

#[async_trait]
impl<E> JobRunner for SyncEmailJobRunner<E>
where
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

impl<E> SyncEmailJobRunner<E>
where
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

            self.keycloak_client
                .update_user_email((*id).into(), email.clone())
                .await?;
        }
        Ok(())
    }
}
