use derive_builder::Builder;
use rust_decimal::Decimal;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "LiquidationProcessId")]
pub enum LiquidationProcessEvent {
    Initialized {
        id: LiquidationProcessId,
        ledger_tx_id: LedgerTxId,
        in_liquidation_account_id: CalaAccountId,
        effective: chrono::NaiveDate,

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
    Completed {},
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct LiquidationProcess {
    pub id: LiquidationProcessId,
    pub ledger_tx_id: LedgerTxId,
    pub obligation_id: ObligationId,
    pub credit_facility_id: CreditFacilityId,
    pub in_liquidation_account_id: CalaAccountId,
    pub initial_amount: UsdCents,
    pub effective: chrono::NaiveDate,
    events: EntityEvents<LiquidationProcessEvent>,
}

impl LiquidationProcess {
    pub fn record_collateral_sent(
        &mut self,
        amount_sent: Satoshis,
        ledger_tx_id: LedgerTxId,
    ) -> Idempotent<()> {
        todo!()
    }

    pub fn record_repayment_received(
        &mut self,
        amount_received: UsdCents,
        ledger_tx_id: LedgerTxId,
    ) -> Idempotent<()> {
        todo!()
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
                    ledger_tx_id,
                    credit_facility_id,
                    in_liquidation_account_id,
                    effective,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .ledger_tx_id(*ledger_tx_id)
                        .credit_facility_id(*credit_facility_id)
                        .in_liquidation_account_id(*in_liquidation_account_id)
                        .effective(*effective)
                }
                LiquidationProcessEvent::CollateralSentOut { .. } => {}
                LiquidationProcessEvent::RepaymentAmountReceived { .. } => {}
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
    pub(crate) ledger_tx_id: LedgerTxId,
    #[builder(setter(into))]
    pub(crate) obligation_id: ObligationId,
    #[builder(setter(into))]
    pub(super) credit_facility_id: CreditFacilityId,
    #[builder(setter(into))]
    pub(super) in_liquidation_account_id: CalaAccountId,
    pub(super) initial_amount: UsdCents,
    pub(super) effective: chrono::NaiveDate,
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
                ledger_tx_id: self.ledger_tx_id,
                credit_facility_id: self.credit_facility_id,
                in_liquidation_account_id: self.in_liquidation_account_id,
                effective: self.effective,
                liquidated_amount: todo!(),
                expected_to_receive: todo!(),
                price_at_initiation: todo!(),
                liquidation_fee: todo!(),
            }],
        )
    }
}
