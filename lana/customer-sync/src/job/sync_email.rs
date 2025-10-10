use async_trait::async_trait;
use futures::StreamExt;
use tracing::instrument;

use core_customer::CoreCustomerEvent;
use keycloak_client::KeycloakClient;
use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use job::*;

#[derive(serde::Serialize)]
pub struct PermanentSyncEmailJobConfig<E> {
    _phantom: std::marker::PhantomData<E>,
}

impl<E> PermanentSyncEmailJobConfig<E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E> JobConfig for PermanentSyncEmailJobConfig<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    type Initializer = PermanentSyncEmailInit<E>;
}

pub struct PermanentSyncEmailInit<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    outbox: Outbox<E>,
    keycloak_client: KeycloakClient,
}

impl<E> PermanentSyncEmailInit<E>
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

const SYNC_EMAIL_JOB: JobType = JobType::new("permanent-sync-email-job");
impl<E> JobInitializer for PermanentSyncEmailInit<E>
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
        Ok(Box::new(PermanentSyncEmailJobRunner::<E> {
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
struct PermanentSyncEmailJobData {
    sequence: outbox::EventSequence,
}

pub struct PermanentSyncEmailJobRunner<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    outbox: Outbox<E>,
    keycloak_client: KeycloakClient,
}

#[async_trait]
impl<E> JobRunner for PermanentSyncEmailJobRunner<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<PermanentSyncEmailJobData>()?
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

impl<E> PermanentSyncEmailJobRunner<E>
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
