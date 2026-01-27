use chrono::{DateTime, Utc};
use core_money::UsdCents;
use core_price::PriceOfOneBTC;
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use es_entity::*;

use crate::{
    PaymentId, RecordProceedsFromLiquidationData,
    primitives::{
        CalaAccountId, CollateralAction, CollateralId, CreditFacilityId, CustodyWalletId,
        LedgerTxId, LiquidationId, PendingCreditFacilityId, Satoshis,
    },
};

use super::{CollateralUpdate, error::CollateralError, liquidation::Liquidation};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "CollateralId")]
pub enum CollateralEvent {
    Initialized {
        id: CollateralId,
        account_id: CalaAccountId,
        credit_facility_id: CreditFacilityId,
        pending_credit_facility_id: PendingCreditFacilityId,

        /// Holds BTC collateral for this credit facility, that is being
        /// liquidated.
        collateral_in_liquidation_account_id: CalaAccountId,
        custody_wallet_id: Option<CustodyWalletId>,
    },
    UpdatedViaManualInput {
        ledger_tx_id: LedgerTxId,
        collateral_amount: Satoshis,
        abs_diff: Satoshis,
        action: CollateralAction,
    },
    UpdatedViaCustodianSync {
        ledger_tx_id: LedgerTxId,
        collateral_amount: Satoshis,
        abs_diff: Satoshis,
        action: CollateralAction,
    },

    /// This Collateral has become subject of liquidation with
    /// `liquidation_id`. It will renmain such until
    /// `ExitedLiquidation` event is emitted.
    EnteredLiquidation {
        liquidation_id: LiquidationId,
    },

    /// Portion of this Collateral was sent to a liquidation with
    /// `liquidation_id`.
    SentToLiquidationViaManualInput {
        amount: Satoshis,
        liquidation_id: LiquidationId,
    },

    ProceedsFromLiquidationReceived {},

    /// This Collateral that has previously been subject of
    /// liquidation with `liquidation_id` is no longer subject of the
    /// liquidation.
    ExitedLiquidation {
        liquidation_id: LiquidationId,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Collateral {
    pub id: CollateralId,
    pub account_id: CalaAccountId,
    pub credit_facility_id: CreditFacilityId,
    pub pending_credit_facility_id: PendingCreditFacilityId,
    pub custody_wallet_id: Option<CustodyWalletId>,
    pub amount: Satoshis,

    /// Ledger account that holds collateral for this entity.
    pub(super) collateral_account_id: CalaAccountId,

    /// Holds parts of collateral of the connected facility, that are
    /// being liquidated.
    collateral_in_liquidation_account_id: CalaAccountId,

    liquidation_proceeds_omnibus_account_id: CalaAccountId,

    /// Holds proceeds received from liquidator for the connected
    /// facility.
    facility_proceeds_from_liquidation_account_id: crate::FacilityProceedsFromLiquidationAccountId,

    /// Holds parts of collateral of the connected facility, that have
    /// already been liquidated.
    liquidated_collateral_account_id: CalaAccountId,

    #[es_entity(nested)]
    #[builder(default)]
    liquidations: Nested<super::liquidation::Liquidation>,

    events: EntityEvents<CollateralEvent>,
}

impl Collateral {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    fn last_liquidation(&self) -> Option<&Liquidation> {
        self.liquidations.iter_persisted().last()
    }

    fn current_liquidation(&mut self) -> Option<LiquidationId> {
        todo!()
    }

    /// Attempts to record that this Collateral has entered a
    /// liquidation with `liquidation_id`.
    ///
    /// # Errors
    ///
    /// Returns `AlreadyInLiquidation` if the Collateral already is in
    /// another liquidation.
    pub fn enter_liquidation(
        &mut self,
        trigger_price: PriceOfOneBTC,
        initially_expected_to_receive: UsdCents,
        initially_estimated_to_liquidate: Satoshis,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CollateralEvent::EnteredLiquidation { .. },
            => CollateralEvent::ExitedLiquidation { .. }
        );

        let liquidation_id = LiquidationId::new();

        self.events
            .push(CollateralEvent::EnteredLiquidation { liquidation_id });

        self.liquidations
            .add_new(super::liquidation::entity::NewLiquidation {
                id: liquidation_id,
                trigger_price,
                initially_expected_to_receive,
                initially_estimated_to_liquidate,
            });

        Idempotent::Executed(())
    }

    /// Attempts to record that this Collateral has exited a
    /// liquidation with `liquidation_id`.
    ///
    /// # Errors
    ///
    /// - `InAnotherLiquidation` if this Collateral is in different liquidation than `liquidation_id`
    /// - `NotInLiquidation` if this Collateral is not in any liquidation
    pub fn exit_liquidation(&mut self) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CollateralEvent::ExitedLiquidation { .. },
            => CollateralEvent::EnteredLiquidation { .. }
        );

        match self.current_liquidation() {
            Some(liquidation_id) => {
                self.events
                    .push(CollateralEvent::ExitedLiquidation { liquidation_id });
                Idempotent::Executed(())
            }
            _ => Idempotent::AlreadyApplied,
        }
    }

    pub fn record_proceeds_from_liquidation(
        &mut self,
        amount_received: UsdCents,
        payment_id: PaymentId,
        ledger_tx_id: LedgerTxId,
    ) -> Idempotent<RecordProceedsFromLiquidationData> {
        let liquidation_id = self.current_liquidation().unwrap();

        let mut liquidation = self
            .liquidations
            .get_persisted_mut(&liquidation_id)
            .unwrap();

        if liquidation
            .record_proceeds_from_liquidation(amount_received, payment_id, ledger_tx_id)
            .did_execute()
        {
            // TODO: Is this needed?
            self.events
                .push(CollateralEvent::ProceedsFromLiquidationReceived {});

            Idempotent::Executed(RecordProceedsFromLiquidationData {
                liquidation_proceeds_omnibus_account_id: self
                    .liquidation_proceeds_omnibus_account_id,
                proceeds_from_liquidation_account_id: self
                    .facility_proceeds_from_liquidation_account_id,
                amount_received,
                collateral_in_liquidation_account_id: self.collateral_in_liquidation_account_id,
                liquidated_collateral_account_id: self.liquidated_collateral_account_id,
                amount_liquidated: liquidation.sent_total,
            })
        } else {
            Idempotent::AlreadyApplied
        }
    }

    pub fn record_collateral_update_via_custodian_sync(
        &mut self,
        new_amount: Satoshis,
        effective: chrono::NaiveDate,
    ) -> Idempotent<CollateralUpdate> {
        let current = self.amount;

        let (abs_diff, action) = match new_amount.cmp(&current) {
            Ordering::Less => (current - new_amount, CollateralAction::Remove),
            Ordering::Greater => (new_amount - current, CollateralAction::Add),
            Ordering::Equal => return Idempotent::AlreadyApplied,
        };

        let tx_id = LedgerTxId::new();

        self.events.push(CollateralEvent::UpdatedViaCustodianSync {
            ledger_tx_id: tx_id,
            abs_diff,
            collateral_amount: new_amount,
            action,
        });

        self.amount = new_amount;

        Idempotent::Executed(CollateralUpdate::Update {
            tx_id,
            abs_diff,
            action,
            effective,
        })
    }

    /// Records that the current value of this Collateral has been
    /// updated to `new_amount` with `effective` date and returns
    /// additional information about this update.
    ///
    /// TODO: Consider whether this should be split into two methods
    /// (liquidation vs. non-liquidation) and branch somewhere higher.
    pub fn record_collateral_update_via_manual_input(
        &mut self,
        new_amount: Satoshis,
        effective: chrono::NaiveDate,
    ) -> Idempotent<CollateralUpdate> {
        let current = self.amount;

        let (abs_diff, action) = match new_amount.cmp(&current) {
            Ordering::Less => (current - new_amount, CollateralAction::Remove),
            Ordering::Greater => (new_amount - current, CollateralAction::Add),
            Ordering::Equal => return Idempotent::AlreadyApplied,
        };

        self.amount = new_amount;
        let tx_id = LedgerTxId::new();

        if action == CollateralAction::Remove
            && let Some(liquidation_id) = self.current_liquidation()
        {
            // This Collateral is currently in liquidation and portion
            // of the amount is being sent out for liquidation.

            self.events
                .push(CollateralEvent::SentToLiquidationViaManualInput {
                    amount: abs_diff,
                    liquidation_id,
                });

            let _ = self
                .liquidations
                .get_persisted_mut(&liquidation_id)
                .expect("")
                .record_collateral_sent_out(abs_diff, tx_id);

            Idempotent::Executed(CollateralUpdate::Liquidation {
                amount: abs_diff,
                effective,
                tx_id,
                collateral_in_liquidation_account_id: self.collateral_in_liquidation_account_id,
            })
        } else {
            self.events.push(CollateralEvent::UpdatedViaManualInput {
                ledger_tx_id: tx_id,
                abs_diff,
                collateral_amount: new_amount,
                action,
            });

            Idempotent::Executed(CollateralUpdate::Update {
                tx_id,
                abs_diff,
                action,
                effective,
            })
        }
    }
}

#[derive(Debug, Builder)]
pub struct NewCollateral {
    #[builder(setter(into))]
    pub(super) id: CollateralId,
    #[builder(setter(into))]
    pub(super) account_id: CalaAccountId,
    #[builder(setter(into))]
    pub(super) credit_facility_id: CreditFacilityId,
    #[builder(setter(into))]
    pub(super) pending_credit_facility_id: PendingCreditFacilityId,
    #[builder(default)]
    pub(super) custody_wallet_id: Option<CustodyWalletId>,
}

impl NewCollateral {
    pub fn builder() -> NewCollateralBuilder {
        NewCollateralBuilder::default()
    }
}

impl TryFromEvents<CollateralEvent> for Collateral {
    fn try_from_events(events: EntityEvents<CollateralEvent>) -> Result<Self, EsEntityError> {
        let mut builder = CollateralBuilder::default();
        for event in events.iter_all() {
            match event {
                CollateralEvent::Initialized {
                    id,
                    credit_facility_id,
                    pending_credit_facility_id,
                    custody_wallet_id,
                    account_id,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .account_id(*account_id)
                        .amount(Satoshis::ZERO)
                        .custody_wallet_id(*custody_wallet_id)
                        .credit_facility_id(*credit_facility_id)
                        .pending_credit_facility_id(*pending_credit_facility_id)
                }
                CollateralEvent::UpdatedViaManualInput {
                    collateral_amount: new_value,
                    ..
                }
                | CollateralEvent::UpdatedViaCustodianSync {
                    collateral_amount: new_value,
                    ..
                } => {
                    builder = builder.amount(*new_value);
                }
                CollateralEvent::EnteredLiquidation { ..
                    // liquidation_id,
                    // collateral_in_liquidation_account_id,
                } => {
                    // builder = builder.current_liquidation(Some(CurrentLiquidation {
                    // liquidation_id: *liquidation_id,
                    // collateral_in_liquidation_account_id: *collateral_in_liquidation_account_id,
                    // }));
                }
                CollateralEvent::ExitedLiquidation { .. } => {
                    // builder = builder.current_liquidation(None);
                }
                CollateralEvent::SentToLiquidationViaManualInput { .. } => {}
            }
        }
        builder.events(events).build()
    }
}

impl IntoEvents<CollateralEvent> for NewCollateral {
    fn into_events(self) -> EntityEvents<CollateralEvent> {
        EntityEvents::init(
            self.id,
            [CollateralEvent::Initialized {
                id: self.id,
                account_id: self.account_id,
                credit_facility_id: self.credit_facility_id,
                pending_credit_facility_id: self.pending_credit_facility_id,
                custody_wallet_id: self.custody_wallet_id,
                collateral_in_liquidation_account_id: todo!(),
            }],
        )
    }
}

#[derive(Copy, Clone)]
struct CurrentLiquidation {
    liquidation_id: LiquidationId,
    collateral_in_liquidation_account_id: CalaAccountId,
}
