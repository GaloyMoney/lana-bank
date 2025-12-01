use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use cala_ledger::AccountId as CalaAccountId;

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
        receivable_account_id: CalaAccountId,
        trigger_price: PriceOfOneBTC,
        initially_expected_to_receive: UsdCents,
        initially_estimated_to_liquidate: Satoshis,
    },
    Updated {
        outstanding: UsdCents,
        to_liquidate_at_current_price: Satoshis,
        current_price: PriceOfOneBTC,
        expected_to_receive: UsdCents,
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
    pub sent_total: Satoshis,
    pub received_total: UsdCents,
    pub receivable_account_id: CalaAccountId,
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
            self.sent_total += amount_sent;

            self.events
                .push(LiquidationProcessEvent::CollateralSentOut {
                    amount: amount_sent,
                    ledger_tx_id,
                });

            Ok(Idempotent::Executed(()))
        }
    }

    pub fn record_repayment_from_liquidation(
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
            self.received_total += amount_received;

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
        if self.received_total >= self.expected_to_receive {
            self.events.push(LiquidationProcessEvent::Satisfied {});
        }
    }

    pub fn complete(&mut self) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            LiquidationProcessEvent::Completed {}
        );

        self.events.push(LiquidationProcessEvent::Completed {});

        Idempotent::Executed(())
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

        let mut amount_sent = Default::default();
        let mut amount_received = Default::default();

        for event in events.iter_all() {
            match event {
                LiquidationProcessEvent::Initialized {
                    id,
                    credit_facility_id,
                    receivable_account_id,
                    initially_expected_to_receive,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .credit_facility_id(*credit_facility_id)
                        .receivable_account_id(*receivable_account_id)
                        .expected_to_receive(*initially_expected_to_receive)
                }
                LiquidationProcessEvent::CollateralSentOut { amount, .. } => {
                    amount_sent += *amount;
                }
                LiquidationProcessEvent::RepaymentAmountReceived { amount, .. } => {
                    amount_received += *amount;
                }
                LiquidationProcessEvent::Satisfied { .. } => {}
                LiquidationProcessEvent::Completed { .. } => {}
                LiquidationProcessEvent::Updated {
                    expected_to_receive,
                    ..
                } => builder = builder.expected_to_receive(*expected_to_receive),
            }
        }

        builder
            .received_total(amount_received)
            .sent_total(amount_sent)
            .events(events)
            .build()
    }
}

#[derive(Debug, Builder)]
pub struct NewLiquidationProcess {
    #[builder(setter(into))]
    pub(crate) id: LiquidationProcessId,
    #[builder(setter(into))]
    pub(crate) credit_facility_id: CreditFacilityId,
    pub(crate) receivable_account_id: CalaAccountId,
    pub(crate) trigger_price: PriceOfOneBTC,
    pub(crate) initially_expected_to_receive: UsdCents,
    pub(crate) initially_estimated_to_liquidate: Satoshis,
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
                receivable_account_id: self.receivable_account_id,
                trigger_price: self.trigger_price,
                initially_expected_to_receive: self.initially_expected_to_receive,
                initially_estimated_to_liquidate: self.initially_estimated_to_liquidate,
            }],
        )
    }
}
