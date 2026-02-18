use core_access::CoreAccessEvent;
use tracing::{Span, instrument};

use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::JobType;
use keycloak_client::KeycloakClient;

pub const USER_ONBOARDING_JOB: JobType = JobType::new("outbox.user-onboarding");

pub struct UserOnboardingHandler {
    keycloak_client: KeycloakClient,
}

impl UserOnboardingHandler {
    pub fn new(keycloak_client: KeycloakClient) -> Self {
        Self { keycloak_client }
    }
}

impl<E> OutboxEventHandler<E> for UserOnboardingHandler
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    #[instrument(name = "user_onboarding.job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        _op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(access_event @ CoreAccessEvent::UserCreated { entity }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", access_event.as_ref());

            self.keycloak_client
                .create_user(entity.email.clone(), entity.id.into())
                .await?;
        }
        Ok(())
    }
}
