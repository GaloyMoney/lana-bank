use tracing::{Span, instrument};

use core_deposit::CoreDepositEvent;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use super::record_last_activity_date::RecordLastActivityDateConfig;

pub const UPDATE_LAST_ACTIVITY_DATE: JobType = JobType::new("outbox.update-last-activity-date");

pub struct UpdateLastActivityDateHandler {
    record_last_activity_date: JobSpawner<RecordLastActivityDateConfig>,
}

impl UpdateLastActivityDateHandler {
    pub fn new(record_last_activity_date: JobSpawner<RecordLastActivityDateConfig>) -> Self {
        Self {
            record_last_activity_date,
        }
    }
}

impl<E> OutboxEventHandler<E> for UpdateLastActivityDateHandler
where
    E: OutboxEventMarker<CoreDepositEvent>,
{
    #[instrument(name = "customer_sync.record_last_activity_date_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (e, deposit_account_id) = match event.as_event() {
            Some(e @ CoreDepositEvent::DepositInitialized { entity }) => {
                (e, entity.deposit_account_id)
            }
            Some(e @ CoreDepositEvent::WithdrawalConfirmed { entity }) => {
                (e, entity.deposit_account_id)
            }
            Some(e @ CoreDepositEvent::DepositReverted { entity }) => {
                (e, entity.deposit_account_id)
            }
            _ => return Ok(()),
        };

        event.inject_trace_parent();
        Span::current().record("handled", true);
        Span::current().record("event_type", e.as_ref());

        self.record_last_activity_date
            .spawn_with_queue_id_in_op(
                op,
                JobId::new(),
                RecordLastActivityDateConfig {
                    deposit_account_id,
                    recorded_at: event.recorded_at,
                },
                deposit_account_id.to_string(),
            )
            .await?;

        Ok(())
    }
}
