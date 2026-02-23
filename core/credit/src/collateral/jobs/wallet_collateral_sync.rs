use tracing::{Span, instrument};

use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use core_custody::CoreCustodyEvent;
use job::{JobId, JobSpawner, JobType};

use super::wallet_collateral_sync_command::WalletCollateralSyncConfig;

pub const WALLET_COLLATERAL_SYNC_JOB: JobType = JobType::new("outbox.wallet-collateral-sync");

pub struct WalletCollateralSyncHandler {
    wallet_collateral_sync_command_spawner: JobSpawner<WalletCollateralSyncConfig>,
}

impl WalletCollateralSyncHandler {
    pub fn new(
        wallet_collateral_sync_command_spawner: JobSpawner<WalletCollateralSyncConfig>,
    ) -> Self {
        Self {
            wallet_collateral_sync_command_spawner,
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

                self.wallet_collateral_sync_command_spawner
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        WalletCollateralSyncConfig {
                            custody_wallet_id: entity.id,
                            updated_collateral: balance.amount,
                            effective: balance.updated_at.date_naive(),
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
