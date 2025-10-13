use async_trait::async_trait;
use core_access::CoreAccessEvent;
use futures::StreamExt;
use tracing::{Span, instrument};

use job::*;

use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use keycloak_client::KeycloakClient;

#[derive(serde::Serialize)]
pub struct UserOnboardingJobConfig<E> {
    _phantom: std::marker::PhantomData<E>,
}
impl<E> UserOnboardingJobConfig<E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<E> JobConfig for UserOnboardingJobConfig<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    type Initializer = UserOnboardingInit<E>;
}

pub struct UserOnboardingInit<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    outbox: Outbox<E>,
    keycloak_client: KeycloakClient,
}

impl<E> UserOnboardingInit<E>
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

const USER_ONBOARDING_JOB: JobType = JobType::new("user-onboarding");
impl<E> JobInitializer for UserOnboardingInit<E>
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
        Ok(Box::new(UserOnboardingJobRunner::<E> {
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
struct UserOnboardingJobData {
    sequence: outbox::EventSequence,
}

pub struct UserOnboardingJobRunner<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    outbox: Outbox<E>,
    keycloak_client: KeycloakClient,
}

impl<E> UserOnboardingJobRunner<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    #[instrument(name = "user_onboarding.job.process_message", parent = None, skip(self, message), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    #[allow(clippy::single_match)]
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match message.as_event() {
            Some(event @ CoreAccessEvent::UserCreated { id, email, .. }) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());

                self.keycloak_client
                    .create_user(email.clone(), id.into())
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[async_trait]
impl<E> JobRunner for UserOnboardingJobRunner<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<UserOnboardingJobData>()?
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
