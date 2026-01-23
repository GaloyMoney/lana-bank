use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use es_entity::*;

use crate::primitives::{
    CalaAccountId, CollateralAction, CollateralId, CreditFacilityId, CustodyWalletId, LedgerTxId,
    LiquidationId, PendingCreditFacilityId, Satoshis,
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
        collateral_in_liquidation_account_id: CalaAccountId,
    },

    /// Portion of this Collateral was sent to a liquidation with
    /// `liquidation_id`.
    SentToLiquidationViaManualInput { liquidation_id: LiquidationId },

    /// This Collateral that has previously been subject of
    /// liquidation with `liquidation_id` is no longer subject of the
    /// liquidation.
    ExitedLiquidation { liquidation_id: LiquidationId },
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
    pub(super) collateral_account_id: crate::CalaAccountId,

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

    pub fn last_liquidation(&self) -> Option<&Liquidation> {
        self.liquidations.iter_persisted().last()
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
        liquidation_id: LiquidationId,
        collateral_in_liquidation_account_id: CalaAccountId,
    ) -> Result<Idempotent<()>, CollateralError> {
        match self.last_liquidation() {
            Some(liquidation) if liquidation.is_ongoing() && liquidation.id == liquidation_id => {
                Ok(Idempotent::AlreadyApplied)
            }
            Some(liquidation) if liquidation.is_ongoing() => {
                Err(CollateralError::AlreadyInLiquidation(liquidation.id))
            }
            _ => {
                self.events.push(CollateralEvent::EnteredLiquidation {
                    liquidation_id,
                    collateral_in_liquidation_account_id,
                });

                Ok(Idempotent::Executed(()))
            }
        }
    }

    /// Attempts to record that this Collateral has exited a
    /// liquidation with `liquidation_id`.
    ///
    /// # Errors
    ///
    /// - `InAnotherLiquidation` if this Collateral is in different liquidation than `liquidation_id`
    /// - `NotInLiquidation` if this Collateral is not in any liquidation
    pub fn exit_liquidation(
        &mut self,
        liquidation_id: LiquidationId,
    ) -> Result<Idempotent<()>, CollateralError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CollateralEvent::ExitedLiquidation { liquidation_id: existing } if *existing == liquidation_id,
            => CollateralEvent::EnteredLiquidation { .. }
        );

        match self.last_liquidation() {
            Some(current) if current.id == liquidation_id => {
                self.events
                    .push(CollateralEvent::ExitedLiquidation { liquidation_id });

                Ok(Idempotent::Executed(()))
            }
            Some(current) => Err(CollateralError::InAnotherLiquidation(current.id)),
            None => Err(CollateralError::NotInLiquidation),
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

        let tx_id = LedgerTxId::new();
        self.amount = new_amount;

        todo!()

        // if action == CollateralAction::Remove
        //     && let Some(CurrentLiquidation {
        //         liquidation_id,
        //         collateral_in_liquidation_account_id,
        //     }) = self.current_liquidation
        // {
        //     self.events
        //         .push(CollateralEvent::SentToLiquidationViaManualInput { liquidation_id });

        //     Idempotent::Executed(CollateralUpdate::Liquidation {
        //         amount: abs_diff,
        //         effective,
        //         tx_id,
        //         collateral_in_liquidation_account_id,
        //     })
        // } else {
        //     self.events.push(CollateralEvent::UpdatedViaManualInput {
        //         ledger_tx_id: tx_id,
        //         abs_diff,
        //         collateral_amount: new_amount,
        //         action,
        //     });

        //     Idempotent::Executed(CollateralUpdate::Update {
        //         tx_id,
        //         abs_diff,
        //         action,
        //         effective,
        //     })
        // }
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
                CollateralEvent::EnteredLiquidation {
                    liquidation_id,
                    collateral_in_liquidation_account_id,
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
            }],
        )
    }
}

#[derive(Copy, Clone)]
struct CurrentLiquidation {
    liquidation_id: LiquidationId,
    collateral_in_liquidation_account_id: CalaAccountId,
}
