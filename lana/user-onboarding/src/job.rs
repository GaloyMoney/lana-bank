use async_trait::async_trait;
use core_access::CoreAccessEvent;
use futures::StreamExt;
use tracing::instrument;

use job::*;

use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use keycloak_client::KeycloakClient;

#[derive(serde::Serialize)]
pub struct PermanentUserOnboardingJobConfig<E> {
    _phantom: std::marker::PhantomData<E>,
}
impl<E> PermanentUserOnboardingJobConfig<E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<E> JobConfig for PermanentUserOnboardingJobConfig<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    type Initializer = PermanentUserOnboardingInit<E>;
}

pub struct PermanentUserOnboardingInit<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    outbox: Outbox<E>,
    keycloak_client: KeycloakClient,
}

impl<E> PermanentUserOnboardingInit<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    pub fn new(outbox: &Outbox<E>, keycloak_client: KeycloakClient) -> Self {
        Self {
            outbox: outbox.clone(),
            keycloak_client,
        }
    }
}

const USER_ONBOARDING_JOB: JobType = JobType::new("permanent-user-onboarding");
impl<E> JobInitializer for PermanentUserOnboardingInit<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        USER_ONBOARDING_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(PermanentUserOnboardingJobRunner::<E> {
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
struct PermanentUserOnboardingJobData {
    sequence: outbox::EventSequence,
}

pub struct PermanentUserOnboardingJobRunner<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    outbox: Outbox<E>,
    keycloak_client: KeycloakClient,
}
#[async_trait]
impl<E> JobRunner for PermanentUserOnboardingJobRunner<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<PermanentUserOnboardingJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;
        while let Some(message) = stream.next().await {
            if let Some(CoreAccessEvent::UserCreated { .. }) = &message.as_ref().as_event() {
                self.handle_create_user(message.as_ref()).await?;
            }

            state.sequence = message.sequence;
            current_job.update_execution_state(state.clone()).await?;
        }

        Ok(JobCompletion::RescheduleNow)
    }
}

impl<E> PermanentUserOnboardingJobRunner<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    #[instrument(name = "user_onboarding.create_keycloak_user", skip(self, message))]
    async fn handle_create_user(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        E: OutboxEventMarker<CoreAccessEvent>,
    {
        if let Some(CoreAccessEvent::UserCreated { id, email, .. }) = message.as_event() {
            message.inject_trace_parent();
            self.keycloak_client
                .create_user(email.clone(), id.into())
                .await?;
        }
        Ok(())
    }
}
