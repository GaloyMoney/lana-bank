use tracing::{Span, instrument};

use core_deposit::CoreDepositEvent;
use es_entity::DbOp;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use super::export_sumsub_deposit::ExportSumsubDepositConfig;
use super::export_sumsub_withdrawal::ExportSumsubWithdrawalConfig;

pub const SUMSUB_EXPORT_JOB: JobType = JobType::new("outbox.sumsub-export");

pub struct SumsubExportHandler {
    export_deposit: JobSpawner<ExportSumsubDepositConfig>,
    export_withdrawal: JobSpawner<ExportSumsubWithdrawalConfig>,
}

impl SumsubExportHandler {
    pub fn new(
        export_deposit: JobSpawner<ExportSumsubDepositConfig>,
        export_withdrawal: JobSpawner<ExportSumsubWithdrawalConfig>,
    ) -> Self {
        Self {
            export_deposit,
            export_withdrawal,
        }
    }
}

impl<E> OutboxEventHandler<E> for SumsubExportHandler
where
    E: OutboxEventMarker<CoreDepositEvent>,
{
    #[instrument(name = "deposit_sync.sumsub_export_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreDepositEvent::DepositInitialized { entity }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.export_deposit
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    ExportSumsubDepositConfig {
                        deposit_account_id: entity.deposit_account_id,
                        deposit_id: entity.id,
                        amount: entity.amount,
                    },
                    entity.deposit_account_id.to_string(),
                )
                .await?;
        }

        if let Some(e @ CoreDepositEvent::WithdrawalConfirmed { entity }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.export_withdrawal
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    ExportSumsubWithdrawalConfig {
                        deposit_account_id: entity.deposit_account_id,
                        withdrawal_id: entity.id,
                        amount: entity.amount,
                    },
                    entity.deposit_account_id.to_string(),
                )
                .await?;
        }

        Ok(())
    }
}
