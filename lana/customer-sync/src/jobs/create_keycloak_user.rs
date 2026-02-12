use async_trait::async_trait;
use futures::StreamExt;
use keycloak_client::KeycloakClient;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use core_customer::CoreCustomerEvent;
use core_deposit::CoreDepositEvent;
use obix::out::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use job::*;

#[derive(Serialize, Deserialize)]
pub(crate) struct CreateKeycloakUserJobConfig<E> {
    _phantom: std::marker::PhantomData<E>,
}

impl<E> Clone for CreateKeycloakUserJobConfig<E> {
    fn clone(&self) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E> CreateKeycloakUserJobConfig<E> {
    pub(crate) fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

pub(crate) struct CreateKeycloakUserInit<E>
where
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    outbox: Outbox<E>,
    keycloak_client: KeycloakClient,
}

impl<E> CreateKeycloakUserInit<E>
where
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    pub(crate) fn new(outbox: &Outbox<E>, keycloak_client: KeycloakClient) -> Self {
        Self {
            outbox: outbox.clone(),
            keycloak_client,
        }
    }
}

const CUSTOMER_SYNC_CREATE_KEYCLOAK_USER: JobType =
    JobType::new("outbox.customer-sync-create-keycloak-user");
impl<E> JobInitializer for CreateKeycloakUserInit<E>
where
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    type Config = CreateKeycloakUserJobConfig<E>;
    fn job_type(&self) -> JobType {
        CUSTOMER_SYNC_CREATE_KEYCLOAK_USER
    }

    fn init(
        &self,
        _: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreateKeycloakUserJobRunner::<E> {
            outbox: self.outbox.clone(),
            keycloak_client: self.keycloak_client.clone(),
        }))
    }

    fn retry_on_error_settings(&self) -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
struct CreateKeycloakUserJobData {
    sequence: obix::EventSequence,
}

pub(crate) struct CreateKeycloakUserJobRunner<E>
where
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    outbox: Outbox<E>,
    keycloak_client: KeycloakClient,
}

impl<E> CreateKeycloakUserJobRunner<E>
where
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    #[instrument(name = "customer_sync.create_keycloak_user_job.process_message", parent = None, skip(self, message), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    #[allow(clippy::single_match)]
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match message.as_event() {
            Some(event @ CoreCustomerEvent::CustomerCreated { entity }) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());

                self.keycloak_client
                    .create_user(entity.email.clone(), entity.id.into())
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[async_trait]
impl<E> JobRunner for CreateKeycloakUserJobRunner<E>
where
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CreateKeycloakUserJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %CUSTOMER_SYNC_CREATE_KEYCLOAK_USER,
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
