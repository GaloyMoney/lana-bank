use tracing::{Span, instrument};

use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use crate::CoreCreditEvent;

use super::record_liquidation::RecordLiquidationConfig;

pub const CREDIT_FACILITY_LIQUIDATIONS_JOB: JobType =
    JobType::new("outbox.credit-facility-liquidations");

pub struct CreditFacilityLiquidationsHandler {
    record_liquidation: JobSpawner<RecordLiquidationConfig>,
}

impl CreditFacilityLiquidationsHandler {
    pub fn new(record_liquidation: JobSpawner<RecordLiquidationConfig>) -> Self {
        Self { record_liquidation }
    }
}

impl<E> OutboxEventHandler<E> for CreditFacilityLiquidationsHandler
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[instrument(name = "outbox.core_credit.collateral_liquidations.process_message_in_op", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreCreditEvent::PartialLiquidationInitiated { entity }) = event.as_event()
        {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            let trigger = entity
                .liquidation_trigger
                .as_ref()
                .expect("liquidation_trigger must be set for PartialLiquidationInitiated");

            self.record_liquidation
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    RecordLiquidationConfig {
                        collateral_id: entity.collateral_id,
                        liquidation_id: trigger.liquidation_id,
                        trigger_price: trigger.trigger_price,
                        initially_expected_to_receive: trigger.initially_expected_to_receive,
                        initially_estimated_to_liquidate: trigger.initially_estimated_to_liquidate,
                        trace_context: Some(tracing_utils::persistence::extract()),
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        Ok(())
    }
}
