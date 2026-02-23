use es_entity::AtomicOperation;
use obix::out::{Outbox, OutboxEventMarker};
use tracing::instrument;

use crate::{
    PublicCollateral,
    collateral::{
        entity::{Collateral, CollateralEvent},
        error::CollateralError,
        public::CoreCreditCollateralEvent,
    },
};

pub struct CollateralPublisher<E>
where
    E: OutboxEventMarker<CoreCreditCollateralEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for CollateralPublisher<E>
where
    E: OutboxEventMarker<CoreCreditCollateralEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> CollateralPublisher<E>
where
    E: obix::out::OutboxEventMarker<CoreCreditCollateralEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    #[instrument(
        name = "credit.collateral.publisher.publish_collateral_in_op",
        skip_all
    )]
    pub async fn publish_collateral_in_op(
        &self,
        op: &mut impl AtomicOperation,
        entity: &Collateral,
        new_events: es_entity::LastPersisted<'_, CollateralEvent>,
    ) -> Result<(), CollateralError> {
        use CollateralEvent::*;
        let events = new_events
            .flat_map(|event| match &event.event {
                UpdatedViaManualInput { .. } | UpdatedViaCustodianSync { .. } => {
                    vec![CoreCreditCollateralEvent::FacilityCollateralUpdated {
                        entity: PublicCollateral::from(entity),
                    }]
                }
                UpdatedViaLiquidation {
                    abs_diff,
                    ledger_tx_id,
                    liquidation_id,
                    ..
                } => vec![
                    CoreCreditCollateralEvent::FacilityCollateralUpdated {
                        entity: PublicCollateral::from(entity),
                    },
                    CoreCreditCollateralEvent::PartialLiquidationCollateralSentOut {
                        liquidation_id: *liquidation_id,
                        secured_loan_id: entity.secured_loan_id,
                        amount: *abs_diff,
                        ledger_tx_id: *ledger_tx_id,
                        recorded_at: event.recorded_at,
                        effective: event.recorded_at.date_naive(),
                    },
                ],
                LiquidationProceedsReceived {
                    liquidation_id,
                    amount,
                    ledger_tx_id,
                    payment_id,
                } => {
                    vec![
                        CoreCreditCollateralEvent::PartialLiquidationProceedsReceived {
                            liquidation_id: *liquidation_id,
                            secured_loan_id: entity.secured_loan_id,
                            amount: *amount,
                            payment_id: *payment_id,
                            ledger_tx_id: *ledger_tx_id,
                            recorded_at: event.recorded_at,
                            effective: event.recorded_at.date_naive(),
                        },
                    ]
                }
                LiquidationCompleted { liquidation_id } => {
                    vec![CoreCreditCollateralEvent::PartialLiquidationCompleted {
                        liquidation_id: *liquidation_id,
                        secured_loan_id: entity.secured_loan_id,
                    }]
                }
                _ => vec![],
            })
            .collect::<Vec<_>>();

        self.outbox.publish_all_persisted(op, events).await?;

        Ok(())
    }
}
