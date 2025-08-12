use async_trait::async_trait;
use core_access::CoreAccessEvent;
use futures::StreamExt;

use job::*;

use outbox::{Outbox, OutboxEventMarker};

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
            if let Some(CoreAccessEvent::UserCreated { id, email, .. }) =
                &message.as_ref().as_event()
            {
                self.keycloak_client
                    .create_user(email.clone(), id.into())
                    .await?;
            }

            state.sequence = message.sequence;
            current_job.update_execution_state(state.clone()).await?;
        }

        Ok(JobCompletion::RescheduleNow)
    }
}
