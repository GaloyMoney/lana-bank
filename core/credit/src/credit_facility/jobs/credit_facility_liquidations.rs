use tracing::{Span, instrument};

use core_credit_collateral::public::CoreCreditCollateralEvent;
use es_entity::DbOp;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use crate::CoreCreditEvent;
use crate::primitives::CreditFacilityId;

use super::record_liquidation_proceeds::RecordLiquidationProceedsConfig;
use super::record_liquidation_started::RecordLiquidationStartedConfig;

pub const CREDIT_FACILITY_LIQUIDATIONS_JOB: JobType =
    JobType::new("outbox.credit-facility-liquidations");

pub struct CreditFacilityLiquidationsHandler {
    record_liquidation_started: JobSpawner<RecordLiquidationStartedConfig>,
    record_liquidation_proceeds: JobSpawner<RecordLiquidationProceedsConfig>,
}

impl CreditFacilityLiquidationsHandler {
    pub fn new(
        record_liquidation_started: JobSpawner<RecordLiquidationStartedConfig>,
        record_liquidation_proceeds: JobSpawner<RecordLiquidationProceedsConfig>,
    ) -> Self {
        Self {
            record_liquidation_started,
            record_liquidation_proceeds,
        }
    }
}

impl<E> OutboxEventHandler<E> for CreditFacilityLiquidationsHandler
where
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollateralEvent>,
{
    #[instrument(name = "outbox.core_credit.credit_facility_liquidations.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut DbOp<'_>,
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
                .ok_or("liquidation_trigger must be set for PartialLiquidationInitiated")?;

            self.record_liquidation_started
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    RecordLiquidationStartedConfig {
                        collateral_id: entity.collateral_id,
                        liquidation_id: trigger.liquidation_id,
                        trigger_price: trigger.trigger_price,
                        initially_expected_to_receive: trigger.initially_expected_to_receive,
                        initially_estimated_to_liquidate: trigger.initially_estimated_to_liquidate,
                    },
                    entity.collateral_id.to_string(),
                )
                .await?;
        }

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
