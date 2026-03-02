use tracing::{Span, instrument};

use core_customer::CoreCustomerEvent;
use job::{JobId, JobSpawner, JobType};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use super::UpdateUserEmailConfig;

pub const SYNC_EMAIL_JOB: JobType = JobType::new("outbox.sync-email-job");

pub struct SyncEmailHandler {
    update_user_email: JobSpawner<UpdateUserEmailConfig>,
}

impl SyncEmailHandler {
    pub fn new(update_user_email: JobSpawner<UpdateUserEmailConfig>) -> Self {
        Self { update_user_email }
    }
}

impl<E> OutboxEventHandler<E> for SyncEmailHandler
where
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    #[instrument(name = "customer_sync.sync_email_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreCustomerEvent::PartyEmailUpdated { entity }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.update_user_email
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    UpdateUserEmailConfig {
                        party_id: entity.id,
                        email: entity.email.clone(),
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        Ok(())
    }
}
