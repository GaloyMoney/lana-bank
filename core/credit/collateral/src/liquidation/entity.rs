use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use cala_ledger::TransactionId as LedgerTxId;
use core_price::PriceOfOneBTC;
use money::{Satoshis, UsdCents};

use es_entity::*;

use crate::{ledger::LiquidationProceedsAccountIds, primitives::*};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "LiquidationId")]
pub enum LiquidationEvent {
    Initialized {
        id: LiquidationId,
        collateral_id: CollateralId,
        trigger_price: PriceOfOneBTC,
        initially_expected_to_receive: UsdCents,
        initially_estimated_to_liquidate: Satoshis,
        account_ids: LiquidationProceedsAccountIds,
    },
    CollateralSentOut {
        amount: Satoshis,
        ledger_tx_id: LedgerTxId,
    },
    ProceedsReceivedAndLiquidationCompleted {
        amount: UsdCents,
        ledger_tx_id: LedgerTxId,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Liquidation {
    pub id: LiquidationId,
    pub collateral_id: CollateralId,
    pub account_ids: LiquidationProceedsAccountIds,
    pub expected_to_receive: UsdCents,
    pub sent_total: Satoshis,
    pub amount_received: UsdCents,

    events: EntityEvents<LiquidationEvent>,
}

impl Liquidation {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub(crate) fn record_collateral_sent_out(
        &mut self,
        amount_sent: Satoshis,
    ) -> Idempotent<LedgerTxId> {
        self.sent_total += amount_sent;

        let ledger_tx_id = LedgerTxId::new();
        self.events.push(LiquidationEvent::CollateralSentOut {
            amount: amount_sent,
            ledger_tx_id,
        });

        Idempotent::Executed(ledger_tx_id)
    }

    pub(crate) fn record_proceeds_from_liquidation_and_complete(
        &mut self,
        amount_received: UsdCents,
    ) -> Idempotent<LedgerTxId> {
        idempotency_guard!(
            self.events.iter_all(),
            LiquidationEvent::ProceedsReceivedAndLiquidationCompleted { .. },
        );

        self.amount_received = amount_received;

        let ledger_tx_id = LedgerTxId::new();
        self.events
            .push(LiquidationEvent::ProceedsReceivedAndLiquidationCompleted {
                amount: amount_received,
                ledger_tx_id,
            });

        Idempotent::Executed(ledger_tx_id)
    }

    pub fn is_completed(&self) -> bool {
        self.events.iter_all().rev().any(|e| {
            matches!(
                e,
                LiquidationEvent::ProceedsReceivedAndLiquidationCompleted { .. }
            )
        })
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
                LiquidationEvent::ProceedsReceivedAndLiquidationCompleted {
                    amount,
                    ledger_tx_id,
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
                    collateral_id,
                    initially_expected_to_receive,
                    account_ids,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .collateral_id(*collateral_id)
                        .account_ids(*account_ids)
                        .expected_to_receive(*initially_expected_to_receive)
                }
                LiquidationEvent::CollateralSentOut { amount, .. } => {
                    amount_sent += *amount;
                }
                LiquidationEvent::ProceedsReceivedAndLiquidationCompleted { amount, .. } => {
                    amount_received = *amount;
                }
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
    pub(crate) collateral_id: CollateralId,
    pub(crate) trigger_price: PriceOfOneBTC,
    pub(crate) initially_expected_to_receive: UsdCents,
    pub(crate) initially_estimated_to_liquidate: Satoshis,
    pub(crate) account_ids: LiquidationProceedsAccountIds,
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
                collateral_id: self.collateral_id,
                trigger_price: self.trigger_price,
                initially_expected_to_receive: self.initially_expected_to_receive,
                initially_estimated_to_liquidate: self.initially_estimated_to_liquidate,
                account_ids: self.account_ids,
            }],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ledger::FacilityProceedsFromLiquidationAccountId, primitives::CalaAccountId};

    fn default_liquidation_proceeds_account_ids() -> LiquidationProceedsAccountIds {
        LiquidationProceedsAccountIds {
            liquidation_proceeds_omnibus_account_id: CalaAccountId::new(),
            proceeds_from_liquidation_account_id: FacilityProceedsFromLiquidationAccountId::new(),
            collateral_in_liquidation_account_id: CalaAccountId::new(),
            liquidated_collateral_account_id: CalaAccountId::new(),
        }
    }

    fn default_new_liquidation() -> NewLiquidation {
        NewLiquidation::builder()
            .id(LiquidationId::new())
            .collateral_id(CollateralId::new())
            .trigger_price(PriceOfOneBTC::new(UsdCents::from(5000000)))
            .initially_expected_to_receive(UsdCents::from(1000))
            .initially_estimated_to_liquidate(Satoshis::from(100000))
            .account_ids(default_liquidation_proceeds_account_ids())
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
        assert!(result.did_execute());
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
        let result = liquidation.record_proceeds_from_liquidation_and_complete(amount);
        assert!(result.did_execute());
        assert_eq!(liquidation.amount_received, amount);

        let result2 = liquidation.record_proceeds_from_liquidation_and_complete(amount);
        assert!(result2.was_already_applied());
        assert_eq!(liquidation.amount_received, amount);
    }

    #[test]
    fn is_completed_returns_false_for_new_liquidation() {
        let liquidation = liquidation_from(default_new_liquidation());
        assert!(!liquidation.is_completed());
    }
}
