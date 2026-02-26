use tracing::{Span, instrument};

use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use core_custody::CoreCustodyEvent;
use job::{JobId, JobSpawner, JobType};

use super::record_collateral_update::RecordCollateralUpdateConfig;

pub const WALLET_COLLATERAL_SYNC_JOB: JobType = JobType::new("outbox.wallet-collateral-sync");

pub struct WalletCollateralSyncHandler {
    record_collateral_update: JobSpawner<RecordCollateralUpdateConfig>,
}

impl WalletCollateralSyncHandler {
    pub fn new(record_collateral_update: JobSpawner<RecordCollateralUpdateConfig>) -> Self {
        Self {
            record_collateral_update,
        }
    }
}

impl<E> OutboxEventHandler<E> for WalletCollateralSyncHandler
where
    E: OutboxEventMarker<CoreCustodyEvent>,
{
    #[instrument(name = "core_credit.wallet_collateral_sync_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        #[allow(clippy::single_match)]
        match event.as_event() {
            Some(e @ CoreCustodyEvent::WalletBalanceUpdated { entity }) => {
                event.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", e.as_ref());

                let balance = entity
                    .balance
                    .as_ref()
                    .expect("WalletBalanceUpdated must have balance");

                self.record_collateral_update
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        RecordCollateralUpdateConfig {
                            custody_wallet_id: entity.id,
                            updated_collateral: balance.amount,
                            effective: balance.updated_at.date_naive(),
                            trace_context: Some(tracing_utils::persistence::extract()),
                        },
                        entity.id.to_string(),
                    )
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}
