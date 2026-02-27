use command_job::CommandJobSpawner;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::JobType;

use core_access::CoreAccessEvent;
use tracing::{Span, instrument};

use super::create_keycloak_user::CreateKeycloakUserCommand;

pub const USER_ONBOARDING_JOB: JobType = JobType::new("outbox.user-onboarding");

pub struct UserOnboardingHandler {
    create_keycloak_user: CommandJobSpawner<CreateKeycloakUserCommand>,
}

impl UserOnboardingHandler {
    pub fn new(create_keycloak_user: CommandJobSpawner<CreateKeycloakUserCommand>) -> Self {
        Self {
            create_keycloak_user,
        }
    }
}

impl<E> OutboxEventHandler<E> for UserOnboardingHandler
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    #[instrument(name = "user_onboarding.job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(access_event @ CoreAccessEvent::UserCreated { entity }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", access_event.as_ref());

            self.create_keycloak_user
                .spawn(
                    op,
                    CreateKeycloakUserCommand {
                        email: entity.email.clone(),
                        user_id: entity.id,
                    },
                )
                .await?;
        }
        Ok(())
    }
}
