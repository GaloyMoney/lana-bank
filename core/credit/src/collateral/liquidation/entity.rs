use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cala_ledger::AccountId as CalaAccountId;
use es_entity::*;

use crate::primitives::*;

use super::{
    FacilityProceedsFromLiquidationAccountId, RecordProceedsFromLiquidationData,
    error::LiquidationError,
};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "LiquidationId")]
pub enum LiquidationEvent {
    Initialized {
        id: LiquidationId,
        credit_facility_id: CreditFacilityId,
        collateral_id: CollateralId,
        liquidation_proceeds_omnibus_account_id: CalaAccountId,
        facility_proceeds_from_liquidation_account_id: FacilityProceedsFromLiquidationAccountId,
        facility_payment_holding_account_id: CalaAccountId,
        facility_uncovered_outstanding_account_id: CalaAccountId,
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
    ProceedsFromLiquidationReceived {
        amount: UsdCents,
        payment_id: PaymentId,
        ledger_tx_id: LedgerTxId,
    },
    Completed {},
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Liquidation {
    pub id: LiquidationId,
    pub credit_facility_id: CreditFacilityId,
    pub collateral_id: CollateralId,
    pub expected_to_receive: UsdCents,
    pub sent_total: Satoshis,
    pub amount_received: UsdCents,
    pub liquidation_proceeds_omnibus_account_id: CalaAccountId,

    /// Holds proceeds received from liquidator for the connected
    /// facility.
    pub facility_proceeds_from_liquidation_account_id: FacilityProceedsFromLiquidationAccountId,

    /// Holds funds meant for payments on the connected facility.
    pub facility_payment_holding_account_id: CalaAccountId,

    /// Holds outstanding not yet covered by an unallocated payment.
    pub facility_uncovered_outstanding_account_id: CalaAccountId,

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
    ) -> Result<Idempotent<LedgerTxId>, LiquidationError> {
        self.sent_total += amount_sent;

        let ledger_tx_id = LedgerTxId::new();
        self.events.push(LiquidationEvent::CollateralSentOut {
            amount: amount_sent,
            ledger_tx_id,
        });

        Ok(Idempotent::Executed(ledger_tx_id))
    }

    pub fn record_proceeds_from_liquidation(
        &mut self,
        amount_received: UsdCents,
        payment_id: PaymentId,
    ) -> Result<Idempotent<RecordProceedsFromLiquidationData>, LiquidationError> {
        idempotency_guard!(
            self.events.iter_all(),
            LiquidationEvent::ProceedsFromLiquidationReceived {
                payment_id: existing_payment_id,
                ..
            } if *existing_payment_id == payment_id,
        );

        self.amount_received = amount_received;

        let ledger_tx_id = LedgerTxId::new();
        self.events
            .push(LiquidationEvent::ProceedsFromLiquidationReceived {
                amount: amount_received,
                payment_id,
                ledger_tx_id,
            });

        Ok(Idempotent::Executed(RecordProceedsFromLiquidationData {
            liquidation_proceeds_omnibus_account_id: self.liquidation_proceeds_omnibus_account_id,
            proceeds_from_liquidation_account_id: self
                .facility_proceeds_from_liquidation_account_id,
            amount_received: self.amount_received,
            collateral_in_liquidation_account_id: self.collateral_in_liquidation_account_id,
            liquidated_collateral_account_id: self.liquidated_collateral_account_id,
            amount_liquidated: self.sent_total,
            ledger_tx_id,
        }))
    }

    pub fn complete(&mut self) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            LiquidationEvent::Completed { .. }
        );

        self.events.push(LiquidationEvent::Completed {});

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

    pub fn proceeds_received(&self) -> Vec<(UsdCents, LedgerTxId)> {
        self.events
            .iter_all()
            .filter_map(|e| match e {
                LiquidationEvent::ProceedsFromLiquidationReceived {
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
                    collateral_id,
                    liquidation_proceeds_omnibus_account_id,
                    facility_proceeds_from_liquidation_account_id,
                    facility_payment_holding_account_id,
                    facility_uncovered_outstanding_account_id,
                    collateral_account_id,
                    collateral_in_liquidation_account_id,
                    liquidated_collateral_account_id,
                    initially_expected_to_receive,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .credit_facility_id(*credit_facility_id)
                        .collateral_id(*collateral_id)
                        .liquidation_proceeds_omnibus_account_id(
                            *liquidation_proceeds_omnibus_account_id,
                        )
                        .facility_proceeds_from_liquidation_account_id(
                            *facility_proceeds_from_liquidation_account_id,
                        )
                        .facility_payment_holding_account_id(*facility_payment_holding_account_id)
                        .facility_uncovered_outstanding_account_id(
                            *facility_uncovered_outstanding_account_id,
                        )
                        .collateral_account_id(*collateral_account_id)
                        .collateral_in_liquidation_account_id(*collateral_in_liquidation_account_id)
                        .liquidated_collateral_account_id(*liquidated_collateral_account_id)
                        .expected_to_receive(*initially_expected_to_receive)
                }
                LiquidationEvent::CollateralSentOut { amount, .. } => {
                    amount_sent += *amount;
                }
                LiquidationEvent::ProceedsFromLiquidationReceived { amount, .. } => {
                    amount_received = *amount;
                }
                LiquidationEvent::Completed { .. } => {}
                LiquidationEvent::Updated {
                    expected_to_receive,
                    ..
                } => builder = builder.expected_to_receive(*expected_to_receive),
            }
        }

        builder
            .amount_received(amount_received)
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
    #[builder(setter(into))]
    pub(crate) collateral_id: CollateralId,
    #[builder(setter(into))]
    pub(crate) liquidation_proceeds_omnibus_account_id: CalaAccountId,
    #[builder(setter(into))]
    pub(crate) facility_proceeds_from_liquidation_account_id:
        FacilityProceedsFromLiquidationAccountId,
    #[builder(setter(into))]
    pub(crate) facility_payment_holding_account_id: CalaAccountId,
    #[builder(setter(into))]
    pub(crate) facility_uncovered_outstanding_account_id: CalaAccountId,
    #[builder(setter(into))]
    pub(crate) collateral_account_id: CalaAccountId,
    #[builder(setter(into))]
    pub(crate) collateral_in_liquidation_account_id: CalaAccountId,
    #[builder(setter(into))]
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
                collateral_id: self.collateral_id,
                liquidation_proceeds_omnibus_account_id: self
                    .liquidation_proceeds_omnibus_account_id,
                facility_proceeds_from_liquidation_account_id: self
                    .facility_proceeds_from_liquidation_account_id,
                facility_payment_holding_account_id: self.facility_payment_holding_account_id,
                facility_uncovered_outstanding_account_id: self
                    .facility_uncovered_outstanding_account_id,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{PriceOfOneBTC, Satoshis, UsdCents};

    fn default_new_liquidation() -> NewLiquidation {
        NewLiquidation::builder()
            .id(LiquidationId::new())
            .credit_facility_id(CreditFacilityId::new())
            .collateral_id(CollateralId::new())
            .liquidation_proceeds_omnibus_account_id(CalaAccountId::new())
            .facility_proceeds_from_liquidation_account_id(
                FacilityProceedsFromLiquidationAccountId::new(),
            )
            .facility_payment_holding_account_id(CalaAccountId::new())
            .facility_uncovered_outstanding_account_id(CalaAccountId::new())
            .collateral_account_id(CalaAccountId::new())
            .collateral_in_liquidation_account_id(CalaAccountId::new())
            .liquidated_collateral_account_id(CalaAccountId::new())
            .trigger_price(PriceOfOneBTC::new(UsdCents::from(5000000)))
            .initially_expected_to_receive(UsdCents::from(1000))
            .initially_estimated_to_liquidate(Satoshis::from(100000))
            .build()
            .unwrap()
    }

    fn liquidation_from(new_liquidation: NewLiquidation) -> Liquidation {
        Liquidation::try_from_events(new_liquidation.into_events()).unwrap()
    }

    #[test]
    fn record_collateral_sent_out_updates_sent_total() {
        let mut liquidation = liquidation_from(default_new_liquidation());
        assert_eq!(liquidation.sent_total, Satoshis::ZERO);

        let amount = Satoshis::from(50000);
        let result = liquidation.record_collateral_sent_out(amount);
        assert!(result.is_ok());
        assert!(result.unwrap().did_execute());
        assert_eq!(liquidation.sent_total, amount);

        let amount2 = Satoshis::from(30000);
        liquidation.record_collateral_sent_out(amount2).unwrap();
        assert_eq!(liquidation.sent_total, amount + amount2);
    }

    #[test]
    fn collateral_sent_out_returns_correct_list() {
        let mut liquidation = liquidation_from(default_new_liquidation());

        let amount1 = Satoshis::from(50000);
        let amount2 = Satoshis::from(30000);
        liquidation.record_collateral_sent_out(amount1).unwrap();
        liquidation.record_collateral_sent_out(amount2).unwrap();

        let sent_out = liquidation.collateral_sent_out();
        assert_eq!(sent_out.len(), 2);
        assert_eq!(sent_out[0].0, amount1);
        assert_eq!(sent_out[1].0, amount2);
    }

    #[test]
    fn record_proceeds_from_liquidation_is_idempotent() {
        let mut liquidation = liquidation_from(default_new_liquidation());

        let amount = UsdCents::from(500);
        let payment_id = PaymentId::new();
        let result = liquidation.record_proceeds_from_liquidation(amount, payment_id);
        assert!(result.is_ok());
        assert!(result.unwrap().did_execute());
        assert_eq!(liquidation.amount_received, amount);

        let result2 = liquidation.record_proceeds_from_liquidation(amount, payment_id);
        assert!(result2.is_ok());
        assert!(result2.unwrap().was_already_applied());
        assert_eq!(liquidation.amount_received, amount);
    }

    #[test]
    fn proceeds_received_returns_correct_list() {
        let mut liquidation = liquidation_from(default_new_liquidation());

        let amount1 = UsdCents::from(500);
        let amount2 = UsdCents::from(300);
        liquidation
            .record_proceeds_from_liquidation(amount1, PaymentId::new())
            .unwrap();
        liquidation
            .record_proceeds_from_liquidation(amount2, PaymentId::new())
            .unwrap();

        let received = liquidation.proceeds_received();
        assert_eq!(received.len(), 2);
        assert_eq!(received[0].0, amount1);
        assert_eq!(received[1].0, amount2);
    }

    #[test]
    fn complete_is_idempotent() {
        let mut liquidation = liquidation_from(default_new_liquidation());
        assert!(!liquidation.is_completed());

        let result = liquidation.complete();
        assert!(result.did_execute());
        assert!(liquidation.is_completed());

        let result2 = liquidation.complete();
        assert!(result2.was_already_applied());
        assert!(liquidation.is_completed());
    }

    #[test]
    fn is_completed_returns_false_for_new_liquidation() {
        let liquidation = liquidation_from(default_new_liquidation());
        assert!(!liquidation.is_completed());
    }
}
