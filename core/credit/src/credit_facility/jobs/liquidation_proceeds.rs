use tracing::{Span, instrument};

use core_credit_collateral::public::CoreCreditCollateralEvent;
use job::{JobId, JobSpawner, JobType};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use crate::primitives::CreditFacilityId;

use super::record_liquidation_proceeds::RecordLiquidationProceedsConfig;

pub const LIQUIDATION_PROCEEDS_JOB: JobType = JobType::new("outbox.liquidation-proceeds");

pub struct RecordLiquidationProceedsHandler {
    record_liquidation_proceeds: JobSpawner<RecordLiquidationProceedsConfig>,
}

impl RecordLiquidationProceedsHandler {
    pub fn new(record_liquidation_proceeds: JobSpawner<RecordLiquidationProceedsConfig>) -> Self {
        Self {
            record_liquidation_proceeds,
        }
    }
}

impl<E> OutboxEventHandler<E> for RecordLiquidationProceedsHandler
where
    E: OutboxEventMarker<CoreCreditCollateralEvent>,
{
    #[instrument(name = "credit.liquidation_proceeds.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(
            e @ CoreCreditCollateralEvent::LiquidationProceedsReceived {
                liquidation_id,
                collateral_id,
                secured_loan_id,
                amount,
                payment_id,
                ..
            },
        ) = event.as_event()
        {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.record_liquidation_proceeds
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    RecordLiquidationProceedsConfig {
                        liquidation_id: *liquidation_id,
                        collateral_id: *collateral_id,
                        credit_facility_id: CreditFacilityId::from(*secured_loan_id),
                        amount: *amount,
                        payment_id: *payment_id,
                    },
                    liquidation_id.to_string(),
                )
                .await?;
        }
        Ok(())
    }
}
