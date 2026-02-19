use keycloak_client::KeycloakClient;
use tracing::{Span, instrument};

use core_customer::CoreCustomerEvent;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::JobType;

pub const SYNC_EMAIL_JOB: JobType = JobType::new("outbox.sync-email-job");

pub struct SyncEmailHandler {
    keycloak_client: KeycloakClient,
}

impl SyncEmailHandler {
    pub fn new(keycloak_client: KeycloakClient) -> Self {
        Self { keycloak_client }
    }
}

impl<E> OutboxEventHandler<E> for SyncEmailHandler
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    #[instrument(name = "customer_sync.sync_email_job.process_message", parent = None, skip(self, _op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        _op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreCustomerEvent::PartyEmailUpdated { entity }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.keycloak_client
                .update_user_email(entity.id.into(), entity.email.clone())
                .await?;
        }
        Ok(())
    }
}
