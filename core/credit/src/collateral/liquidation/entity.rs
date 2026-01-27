use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "LiquidationId")]
pub enum LiquidationEvent {
    Initialized {
        id: LiquidationId,
        trigger_price: PriceOfOneBTC,
        initially_expected_to_receive: UsdCents,
        initially_estimated_to_liquidate: Satoshis,
    },
    // Updated {
    // outstanding: UsdCents,
    // to_liquidate_at_current_price: Satoshis,
    // current_price: PriceOfOneBTC,
    // expected_to_receive: UsdCents,
    // },
    /// Portion of collateral (`amount`) has been sent to
    /// liquidation. This operation can be repeated multiple
    /// times. This movement of funds has been recorded on ledger in
    /// Transaction with `ledger_tx_id`.
    CollateralSentOut {
        amount: Satoshis,
        ledger_tx_id: LedgerTxId,
    },

    /// Proceeds from liquidation has been received and therefore,
    /// since only one receival of the proceeds is expected, this
    /// Liquidation has been completed. No other operations on it are
    /// allowed.
    ProceedsFromLiquidationReceived {
        /// Amount of fiat received from liquidation.
        amount: UsdCents,

        /// ID of Payment that will be used to further process the
        /// proceeds.
        payment_id: PaymentId,

        /// ID of Transaction which records this movement of funds.
        ledger_tx_id: LedgerTxId,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Liquidation {
    pub id: LiquidationId,
    // pub expected_to_receive: UsdCents,
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

    pub fn record_collateral_sent_out(
        &mut self,
        amount_sent: Satoshis,
        ledger_tx_id: LedgerTxId,
    ) -> Idempotent<()> {
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

        Idempotent::Executed(())
    }

    pub fn record_proceeds_from_liquidation(
        &mut self,
        amount_received: UsdCents,
        payment_id: PaymentId,
        ledger_tx_id: LedgerTxId,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            LiquidationEvent::ProceedsFromLiquidationReceived { .. }
        );

        self.amount_received = amount_received;

        self.events
            .push(LiquidationEvent::ProceedsFromLiquidationReceived {
                amount: amount_received,
                payment_id,
                ledger_tx_id,
            });

        Idempotent::Executed(())
    }

    // pub fn complete(&mut self) -> Idempotent<()> {
    //     idempotency_guard!(
    //         self.events.iter_all().rev(),
    //         LiquidationEvent::Completed { .. }
    //     );

    //     self.events.push(LiquidationEvent::Completed {});

    //     Idempotent::Executed(())
    // }

    // pub fn is_ongoing(&self) -> bool {
    //     !self.is_completed()
    // }

    // pub fn is_completed(&self) -> bool {
    //     self.events
    //         .iter_all()
    //         .rev()
    //         .any(|e| matches!(e, LiquidationEvent::Completed { .. }))
    // }

    // pub fn collateral_sent_out(&self) -> Vec<(Satoshis, LedgerTxId)> {
    //     self.events
    //         .iter_all()
    //         .filter_map(|e| match e {
    //             LiquidationEvent::CollateralSentOut {
    //                 amount,
    //                 ledger_tx_id,
    //             } => Some((*amount, *ledger_tx_id)),
    //             _ => None,
    //         })
    //         .collect()
    // }

    // pub fn proceeds_received(&self) -> Vec<(UsdCents, LedgerTxId)> {
    //     self.events
    //         .iter_all()
    //         .filter_map(|e| match e {
    //             LiquidationEvent::ProceedsFromLiquidationReceived {
    //                 amount,
    //                 ledger_tx_id,
    //                 ..
    //             } => Some((*amount, *ledger_tx_id)),
    //             _ => None,
    //         })
    //         .collect()
    // }
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
                    // initially_expected_to_receive,
                    ..
                } => {
                    builder = builder.id(*id)
                    // .expected_to_receive(*initially_expected_to_receive)
                }
                LiquidationEvent::CollateralSentOut { amount, .. } => {
                    amount_sent += *amount;
                }
                LiquidationEvent::ProceedsFromLiquidationReceived { amount, .. } => {
                    amount_received = *amount;
                } // LiquidationEvent::Completed { .. } => {}
                  // LiquidationEvent::Updated {
                  // expected_to_receive,
                  // ..
                  // } => builder = builder.expected_to_receive(*expected_to_receive),
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
                trigger_price: self.trigger_price,
                initially_expected_to_receive: self.initially_expected_to_receive,
                initially_estimated_to_liquidate: self.initially_estimated_to_liquidate,
            }],
        )
    }
}
