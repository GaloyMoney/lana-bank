use tracing::{Span, instrument};

use core_customer::CoreCustomerEvent;
use job::{JobId, JobSpawner, JobType};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use super::DisableKeycloakUserConfig;

pub const CUSTOMER_SYNC_DISABLE_KEYCLOAK_USER: JobType =
    JobType::new("outbox.customer-sync-disable-keycloak-user");

pub struct SyncCustomerCloseKeycloakHandler {
    disable_keycloak_user: JobSpawner<DisableKeycloakUserConfig>,
}

impl SyncCustomerCloseKeycloakHandler {
    pub fn new(disable_keycloak_user: JobSpawner<DisableKeycloakUserConfig>) -> Self {
        Self {
            disable_keycloak_user,
        }
    }
}

impl<E> OutboxEventHandler<E> for SyncCustomerCloseKeycloakHandler
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    #[instrument(name = "customer_sync.disable_keycloak_user_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreCustomerEvent::CustomerClosed { entity }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.disable_keycloak_user
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    DisableKeycloakUserConfig {
                        party_id: entity.party_id,
                    },
                    entity.party_id.to_string(),
                )
                .await?;
        }
        Ok(())
    }
}
