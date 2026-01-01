use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cala_ledger::AccountId as CalaAccountId;
use es_entity::*;

use crate::primitives::*;

use super::{
    FacilityLiquidationInHoldingAccount, RecordPaymentFromLiquidationData, error::LiquidationError,
};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "LiquidationId")]
pub enum LiquidationEvent {
    Initialized {
        id: LiquidationId,
        credit_facility_id: CreditFacilityId,
        liquidation_omnibus_account_id: CalaAccountId,
        facility_liquidation_in_holding_account_id: FacilityLiquidationInHoldingAccount,
        facility_payment_holding_account_id: CalaAccountId,
        collateral_account_id: CalaAccountId,
        collateral_in_liquidation_account_id: CalaAccountId,
        liquidated_collateral_account_id: CalaAccountId,
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
        payment_id: PaymentId,
        ledger_tx_id: LedgerTxId,
    },
    Completed {
        payment_id: PaymentId,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Liquidation {
    pub id: LiquidationId,
    pub credit_facility_id: CreditFacilityId,
    pub expected_to_receive: UsdCents,
    pub sent_total: Satoshis,
    pub received_total: UsdCents,
    pub liquidation_omnibus_account_id: CalaAccountId,

    /// Holds payments received from liquidator for the connected
    /// facility.
    pub facility_liquidation_in_holding_account_id: FacilityLiquidationInHoldingAccount,

    /// Holds funds meant for payments on the connected facility.
    pub facility_payment_holding_account_id: CalaAccountId,

    /// Holds collateral of the connected facility.
    pub collateral_account_id: CalaAccountId,

    /// Holds parts of collateral of the connected facility, that are
    /// being liquidated.
    pub collateral_in_liquidation_account_id: CalaAccountId,

    /// Holds parts of collateral of the connected facility, that have
    /// already been liquidated.
    pub liquidated_collateral_account_id: CalaAccountId,

    events: EntityEvents<LiquidationEvent>,
}

impl Liquidation {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn record_collateral_sent_out(
        &mut self,
        amount_sent: Satoshis,
        ledger_tx_id: LedgerTxId,
    ) -> Result<Idempotent<()>, LiquidationError> {
        idempotency_guard!(
            self.events.iter_all(),
            LiquidationEvent::CollateralSentOut {
                amount,
                ledger_tx_id: tx_id
            } if amount_sent == *amount && ledger_tx_id == *tx_id
        );

        self.sent_total += amount_sent;

        self.events.push(LiquidationEvent::CollateralSentOut {
            amount: amount_sent,
            ledger_tx_id,
        });

        Ok(Idempotent::Executed(()))
    }

    pub fn record_repayment_from_liquidation(
        &mut self,
        amount_received: UsdCents,
        payment_id: PaymentId,
        ledger_tx_id: LedgerTxId,
    ) -> Result<Idempotent<RecordPaymentFromLiquidationData>, LiquidationError> {
        idempotency_guard!(
            self.events.iter_all(),
            LiquidationEvent::RepaymentAmountReceived {
                payment_id: id,..
            } if payment_id == *id
        );

        self.received_total += amount_received;

        self.events.push(LiquidationEvent::RepaymentAmountReceived {
            amount: amount_received,
            payment_id,
            ledger_tx_id,
        });

        Ok(Idempotent::Executed(RecordPaymentFromLiquidationData {
            liquidation_omnibus_account_id: self.liquidation_omnibus_account_id,
            liquidation_in_holding_account_id: self.facility_liquidation_in_holding_account_id,
            amount_received: self.received_total,
            collateral_in_liquidation_account_id: self.collateral_in_liquidation_account_id,
            liquidated_collateral_account_id: self.liquidated_collateral_account_id,
            amount_liquidated: self.sent_total,
        }))
    }

    pub fn complete(&mut self, payment_id: PaymentId) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            LiquidationEvent::Completed { .. }
        );

        self.events.push(LiquidationEvent::Completed { payment_id });

        Idempotent::Executed(())
    }

    pub fn is_completed(&self) -> bool {
        self.events
            .iter_all()
            .rev()
            .any(|e| matches!(e, LiquidationEvent::Completed { .. }))
    }

    pub fn collateral_sent_out(&self) -> Vec<(Satoshis, LedgerTxId)> {
        self.events
            .iter_all()
            .filter_map(|e| match e {
                LiquidationEvent::CollateralSentOut {
                    amount,
                    ledger_tx_id,
                } => Some((*amount, *ledger_tx_id)),
                _ => None,
            })
            .collect()
    }

    pub fn repayment_amounts_received(&self) -> Vec<(UsdCents, LedgerTxId)> {
        self.events
            .iter_all()
            .filter_map(|e| match e {
                LiquidationEvent::RepaymentAmountReceived {
                    amount,
                    ledger_tx_id,
                    ..
                } => Some((*amount, *ledger_tx_id)),
                _ => None,
            })
            .collect()
    }
}

impl TryFromEvents<LiquidationEvent> for Liquidation {
    fn try_from_events(events: EntityEvents<LiquidationEvent>) -> Result<Self, EsEntityError> {
        let mut builder = LiquidationBuilder::default();

        let mut amount_sent = Default::default();
        let mut amount_received = Default::default();

        for event in events.iter_all() {
            match event {
                LiquidationEvent::Initialized {
                    id,
                    credit_facility_id,
                    liquidation_omnibus_account_id,
                    facility_liquidation_in_holding_account_id,
                    facility_payment_holding_account_id,
                    collateral_account_id,
                    collateral_in_liquidation_account_id,
                    liquidated_collateral_account_id,
                    initially_expected_to_receive,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .credit_facility_id(*credit_facility_id)
                        .liquidation_omnibus_account_id(*liquidation_omnibus_account_id)
                        .facility_liquidation_in_holding_account_id(
                            *facility_liquidation_in_holding_account_id,
                        )
                        .facility_payment_holding_account_id(*facility_payment_holding_account_id)
                        .collateral_account_id(*collateral_account_id)
                        .collateral_in_liquidation_account_id(*collateral_in_liquidation_account_id)
                        .liquidated_collateral_account_id(*liquidated_collateral_account_id)
                        .expected_to_receive(*initially_expected_to_receive)
                }
                LiquidationEvent::CollateralSentOut { amount, .. } => {
                    amount_sent += *amount;
                }
                LiquidationEvent::RepaymentAmountReceived { amount, .. } => {
                    amount_received += *amount;
                }
                LiquidationEvent::Completed { .. } => {}
                LiquidationEvent::Updated {
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
pub struct NewLiquidation {
    #[builder(setter(into))]
    pub(crate) id: LiquidationId,
    #[builder(setter(into))]
    pub(crate) credit_facility_id: CreditFacilityId,
    pub(crate) liquidation_omnibus_account_id: CalaAccountId,
    pub(crate) facility_liquidation_in_holding_account_id: FacilityLiquidationInHoldingAccount,
    pub(crate) facility_payment_holding_account_id: CalaAccountId,
    pub(crate) collateral_account_id: CalaAccountId,
    pub(crate) collateral_in_liquidation_account_id: CalaAccountId,
    pub(crate) liquidated_collateral_account_id: CalaAccountId,
    pub(crate) trigger_price: PriceOfOneBTC,
    pub(crate) initially_expected_to_receive: UsdCents,
    pub(crate) initially_estimated_to_liquidate: Satoshis,
}

impl NewLiquidation {
    pub fn builder() -> NewLiquidationBuilder {
        NewLiquidationBuilder::default()
    }
}

impl IntoEvents<LiquidationEvent> for NewLiquidation {
    fn into_events(self) -> EntityEvents<LiquidationEvent> {
        EntityEvents::init(
            self.id,
            [LiquidationEvent::Initialized {
                id: self.id,
                credit_facility_id: self.credit_facility_id,
                liquidation_omnibus_account_id: self.liquidation_omnibus_account_id,
                facility_liquidation_in_holding_account_id: self
                    .facility_liquidation_in_holding_account_id,
                facility_payment_holding_account_id: self.facility_payment_holding_account_id,
                trigger_price: self.trigger_price,
                initially_expected_to_receive: self.initially_expected_to_receive,
                initially_estimated_to_liquidate: self.initially_estimated_to_liquidate,
                collateral_account_id: self.collateral_account_id,
                collateral_in_liquidation_account_id: self.collateral_in_liquidation_account_id,
                liquidated_collateral_account_id: self.liquidated_collateral_account_id,
            }],
        )
    }
}
