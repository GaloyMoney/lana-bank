use tracing::{Span, instrument};

use core_credit_collateral::public::CoreCreditCollateralEvent;
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
}

impl CreditFacilityLiquidationsHandler {
    pub fn new(record_liquidation_started: JobSpawner<RecordLiquidationStartedConfig>) -> Self {
        Self {
            record_liquidation_started,
        }
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
        Ok(())
    }
}

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
