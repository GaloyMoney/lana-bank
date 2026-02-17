use core_access::CoreAccessEvent;
use tracing::{Span, instrument};

use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use keycloak_client::KeycloakClient;

pub struct UserOnboardingHandler {
    pub(super) keycloak_client: KeycloakClient,
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
        match event.as_event() {
            Some(access_event @ CoreAccessEvent::UserCreated { entity }) => {
                event.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", access_event.as_ref());

                self.keycloak_client
                    .create_user(entity.email.clone(), entity.id.into())
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}
