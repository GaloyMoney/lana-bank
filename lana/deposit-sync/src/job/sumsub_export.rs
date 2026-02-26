use tracing::{Span, instrument};

use core_deposit::CoreDepositEvent;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use super::export_to_sumsub::ExportToSumsubConfig;

pub const SUMSUB_EXPORT_JOB: JobType = JobType::new("outbox.sumsub-export");

pub struct SumsubExportHandler {
    export_to_sumsub: JobSpawner<ExportToSumsubConfig>,
}

impl SumsubExportHandler {
    pub fn new(export_to_sumsub: JobSpawner<ExportToSumsubConfig>) -> Self {
        Self { export_to_sumsub }
    }
}

impl<E> OutboxEventHandler<E> for SumsubExportHandler
where
    E: OutboxEventMarker<CoreDepositEvent>,
{
    #[instrument(name = "deposit_sync.sumsub_export_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match event.as_event() {
            Some(e @ CoreDepositEvent::DepositInitialized { entity }) => {
                event.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", e.as_ref());

                self.export_to_sumsub
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        ExportToSumsubConfig::Deposit {
                            id: entity.id,
                            deposit_account_id: entity.deposit_account_id,
                            amount: entity.amount,
                        },
                        entity.deposit_account_id.to_string(),
                    )
                    .await?;
            }
            Some(e @ CoreDepositEvent::WithdrawalConfirmed { entity }) => {
                event.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", e.as_ref());

                self.export_to_sumsub
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        ExportToSumsubConfig::Withdrawal {
                            id: entity.id,
                            deposit_account_id: entity.deposit_account_id,
                            amount: entity.amount,
                        },
                        entity.deposit_account_id.to_string(),
                    )
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}
