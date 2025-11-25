use derive_builder::Builder;
use rust_decimal::Decimal;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

use super::error::LiquidationProcessError;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "LiquidationProcessId")]
pub enum LiquidationProcessEvent {
    Initialized {
        id: LiquidationProcessId,
        credit_facility_id: CreditFacilityId,
        liquidated_amount: Satoshis,
        expected_to_receive: UsdCents,
        price_at_initiation: PriceOfOneBTC,
        liquidation_fee: Decimal,
    },
    CollateralSentOut {
        amount: Satoshis,
        ledger_tx_id: LedgerTxId,
    },
    RepaymentAmountReceived {
        amount: UsdCents,
        ledger_tx_id: LedgerTxId,
    },
    Satisfied {},
    Completed {},
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct LiquidationProcess {
    pub id: LiquidationProcessId,
    pub credit_facility_id: CreditFacilityId,
    pub expected_to_receive: UsdCents,
    pub amount_sent: Satoshis,
    pub amount_received: UsdCents,
    events: EntityEvents<LiquidationProcessEvent>,
}

impl LiquidationProcess {
    pub fn record_collateral_sent_out(
        &mut self,
        amount_sent: Satoshis,
        ledger_tx_id: LedgerTxId,
    ) -> Result<Idempotent<()>, LiquidationProcessError> {
        idempotency_guard!(
            self.events.iter_all(),
            LiquidationProcessEvent::CollateralSentOut {
                amount,
                ledger_tx_id: tx_id
            } if amount_sent == *amount && ledger_tx_id == *tx_id
        );

        if self.is_satisfied() {
            Err(LiquidationProcessError::AlreadySatisfied)
        } else {
            self.amount_sent += amount_sent;

            self.events
                .push(LiquidationProcessEvent::CollateralSentOut {
                    amount: amount_sent,
                    ledger_tx_id,
                });

            Ok(Idempotent::Executed(()))
        }
    }

    pub fn record_repayment_from_liquidator(
        &mut self,
        amount_received: UsdCents,
        ledger_tx_id: LedgerTxId,
    ) -> Result<Idempotent<()>, LiquidationProcessError> {
        idempotency_guard!(
            self.events.iter_all(),
            LiquidationProcessEvent::RepaymentAmountReceived {
                amount,
                ledger_tx_id: tx_id
            } if amount_received == *amount && ledger_tx_id == *tx_id
        );

        if self.is_satisfied() {
            Err(LiquidationProcessError::AlreadySatisfied)
        } else {
            self.amount_received += amount_received;

            self.events
                .push(LiquidationProcessEvent::RepaymentAmountReceived {
                    amount: amount_received,
                    ledger_tx_id,
                });

            self.mark_satisfied_if_needed();

            Ok(Idempotent::Executed(()))
        }
    }

    fn mark_satisfied_if_needed(&mut self) {
        if self.amount_received >= self.expected_to_receive {
            self.events.push(LiquidationProcessEvent::Satisfied {});
        }
    }

    pub fn is_satisfied(&self) -> bool {
        self.events
            .iter_all()
            .rev()
            .any(|e| matches!(e, LiquidationProcessEvent::Satisfied { .. }))
    }
}

impl TryFromEvents<LiquidationProcessEvent> for LiquidationProcess {
    fn try_from_events(
        events: EntityEvents<LiquidationProcessEvent>,
    ) -> Result<Self, EsEntityError> {
        let mut builder = LiquidationProcessBuilder::default();
        for event in events.iter_all() {
            match event {
                LiquidationProcessEvent::Initialized {
                    id,
                    credit_facility_id,
                    ..
                } => builder = builder.id(*id).credit_facility_id(*credit_facility_id),
                LiquidationProcessEvent::CollateralSentOut { .. } => {}
                LiquidationProcessEvent::RepaymentAmountReceived { .. } => {}
                LiquidationProcessEvent::Satisfied { .. } => {}
                LiquidationProcessEvent::Completed { .. } => {}
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewLiquidationProcess {
    #[builder(setter(into))]
    pub(crate) id: LiquidationProcessId,
    #[builder(setter(into))]
    pub(crate) credit_facility_id: CreditFacilityId,
}

impl NewLiquidationProcess {
    pub fn builder() -> NewLiquidationProcessBuilder {
        NewLiquidationProcessBuilder::default()
    }
}

impl IntoEvents<LiquidationProcessEvent> for NewLiquidationProcess {
    fn into_events(self) -> EntityEvents<LiquidationProcessEvent> {
        EntityEvents::init(
            self.id,
            [LiquidationProcessEvent::Initialized {
                id: self.id,
                credit_facility_id: self.credit_facility_id,
                liquidated_amount: todo!(),
                expected_to_receive: todo!(),
                price_at_initiation: todo!(),
                liquidation_fee: todo!(),
            }],
        )
    }
}
