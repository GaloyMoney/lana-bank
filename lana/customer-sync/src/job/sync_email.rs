use async_trait::async_trait;
use futures::StreamExt;
use tokio::select;
use tracing::{Span, instrument};

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

const SYNC_EMAIL_JOB: JobType = JobType::new("outbox.sync-email-job");
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

impl<E> SyncEmailJobRunner<E>
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    #[instrument(name = "customer_sync.sync_email_job.process_message", parent = None, skip(self, message), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    #[allow(clippy::single_match)]
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match message.as_event() {
            Some(event @ CoreCustomerEvent::CustomerEmailUpdated { id, email }) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());

                self.keycloak_client
                    .update_user_email((*id).into(), email.clone())
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
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

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %SYNC_EMAIL_JOB,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(message) => {
                            self.process_message(message.as_ref()).await?;
                            state.sequence = message.sequence;
                            current_job.update_execution_state(&state).await?;
                        }
                        None => {
                            return Ok(JobCompletion::RescheduleNow);
                        }
                    }
                }
            }
        }
    }
}
