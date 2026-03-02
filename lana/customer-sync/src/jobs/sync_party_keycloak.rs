use tracing::{Span, instrument};

use core_customer::CoreCustomerEvent;
use job::{JobId, JobSpawner, JobType};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use super::CreateKeycloakUserConfig;

pub const CUSTOMER_SYNC_CREATE_KEYCLOAK_USER: JobType =
    JobType::new("outbox.customer-sync-create-keycloak-user");

pub struct SyncPartyKeycloakHandler {
    create_keycloak_user: JobSpawner<CreateKeycloakUserConfig>,
}

impl SyncPartyKeycloakHandler {
    pub fn new(create_keycloak_user: JobSpawner<CreateKeycloakUserConfig>) -> Self {
        Self {
            create_keycloak_user,
        }
    }
}

impl<E> OutboxEventHandler<E> for SyncPartyKeycloakHandler
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    #[instrument(name = "customer_sync.create_keycloak_user_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreCustomerEvent::PartyCreated { entity }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.create_keycloak_user
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    CreateKeycloakUserConfig {
                        email: entity.email.clone(),
                        party_id: entity.id,
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        Ok(())
    }
}
