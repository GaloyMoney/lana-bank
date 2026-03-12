use tracing::{Span, instrument};

use core_credit_collateral::public::CoreCreditCollateralEvent;
use job::{JobId, JobSpawner, JobType};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use crate::primitives::CreditFacilityId;

use super::liquidation_payment::LiquidationPaymentJobConfig;

pub const SPAWN_LIQUIDATION_PAYMENT_JOB: JobType = JobType::new("outbox.spawn-liquidation-payment");

pub struct SpawnLiquidationPaymentHandler<E> {
    liquidation_payment: JobSpawner<LiquidationPaymentJobConfig<E>>,
}

impl<E> SpawnLiquidationPaymentHandler<E> {
    pub fn new(liquidation_payment: JobSpawner<LiquidationPaymentJobConfig<E>>) -> Self {
        Self {
            liquidation_payment,
        }
    }
}

impl<E> OutboxEventHandler<E> for SpawnLiquidationPaymentHandler<E>
where
    E: OutboxEventMarker<CoreCreditCollateralEvent>,
{
    #[instrument(name = "credit.spawn_liquidation_payment.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(
            e @ CoreCreditCollateralEvent::LiquidationStarted {
                liquidation_id,
                collateral_id,
                secured_loan_id,
            },
        ) = event.as_event()
        {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.liquidation_payment
                .spawn_in_op(
                    op,
                    JobId::new(),
                    LiquidationPaymentJobConfig::<E> {
                        liquidation_id: *liquidation_id,
                        collateral_id: *collateral_id,
                        credit_facility_id: CreditFacilityId::from(*secured_loan_id),
                        _phantom: std::marker::PhantomData,
                    },
                )
                .await?;
        }
        Ok(())
    }
}
