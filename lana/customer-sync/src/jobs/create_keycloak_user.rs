use tracing::{Span, instrument};

use core_customer::CoreCustomerEvent;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use super::create_customer_keycloak_user::CreateCustomerKeycloakUserConfig;

pub const CUSTOMER_SYNC_CREATE_KEYCLOAK_USER: JobType =
    JobType::new("outbox.customer-sync-create-keycloak-user");

pub struct CreateKeycloakUserHandler {
    create_customer_keycloak_user: JobSpawner<CreateCustomerKeycloakUserConfig>,
}

impl CreateKeycloakUserHandler {
    pub fn new(
        create_customer_keycloak_user: JobSpawner<CreateCustomerKeycloakUserConfig>,
    ) -> Self {
        Self {
            create_customer_keycloak_user,
        }
    }
}

impl<E> OutboxEventHandler<E> for CreateKeycloakUserHandler
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

            self.create_customer_keycloak_user
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    CreateCustomerKeycloakUserConfig {
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
