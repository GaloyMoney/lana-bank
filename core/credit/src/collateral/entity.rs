use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use es_entity::*;

use crate::{
    CollateralLedgerAccountIds,
    liquidation::{Liquidation, NewLiquidation, RecordProceedsFromLiquidationData},
    primitives::{
        CalaAccountId, CollateralAction, CollateralId, CreditFacilityId, CustodyWalletId,
        LedgerTxId, LiquidationId, PaymentId, PendingCreditFacilityId, PriceOfOneBTC, Satoshis,
        UsdCents,
    },
};

use super::{CollateralUpdate, SendCollateralToLiquidationResult, error::CollateralError};

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
        account_ids: CollateralLedgerAccountIds,
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
    UpdatedViaLiquidation {
        liquidation_id: LiquidationId,
        collateral_amount: Satoshis,
        abs_diff: Satoshis,
        action: CollateralAction,
    },
    LiquidationStarted {
        liquidation_id: LiquidationId,
        trigger_price: PriceOfOneBTC,
        initially_expected_to_receive: UsdCents,
        initially_estimated_to_liquidate: Satoshis,
    },
    LiquidationCompleted {
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

    pub(crate) account_ids: CollateralLedgerAccountIds,

    #[es_entity(nested)]
    #[builder(default)]
    liquidations: Nested<Liquidation>,

    events: EntityEvents<CollateralEvent>,
}

impl Collateral {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
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

        Idempotent::Executed(CollateralUpdate {
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

        self.events.push(CollateralEvent::UpdatedViaManualInput {
            ledger_tx_id: tx_id,
            abs_diff,
            collateral_amount: new_amount,
            action,
        });

        self.amount = new_amount;

        Idempotent::Executed(CollateralUpdate {
            tx_id,
            abs_diff,
            action,
            effective,
        })
    }

    /// Sends collateral to liquidation, updating both the nested Liquidation entity
    /// and the Collateral's own amount. This is the single entry point for sending
    /// collateral to liquidation through the Collateral aggregate.
    ///
    /// Returns the data needed for ledger posting.
    pub fn send_collateral_to_liquidation(
        &mut self,
        liquidation_id: LiquidationId,
        amount: Satoshis,
        effective: chrono::NaiveDate,
    ) -> Result<Idempotent<SendCollateralToLiquidationResult>, CollateralError> {
        if amount == Satoshis::ZERO {
            return Ok(Idempotent::AlreadyApplied);
        }

        if amount > self.amount {
            return Err(CollateralError::InsufficientCollateral {
                requested: amount,
                available: self.amount,
            });
        }

        // Get the nested liquidation
        let liquidation = self
            .liquidations
            .get_persisted_mut(&liquidation_id)
            .ok_or(CollateralError::LiquidationNotFound(liquidation_id))?;

        // Generate a single tx_id for both operations
        let ledger_tx_id = LedgerTxId::new();

        // Update the nested Liquidation entity
        match liquidation.record_collateral_sent_out(amount, ledger_tx_id)? {
            Idempotent::AlreadyApplied => return Ok(Idempotent::AlreadyApplied),
            Idempotent::Executed(()) => {}
        }

        // Update Collateral's own amount
        let new_amount = self.amount - amount;
        self.events.push(CollateralEvent::UpdatedViaLiquidation {
            liquidation_id,
            abs_diff: amount,
            collateral_amount: new_amount,
            action: CollateralAction::Remove,
        });
        self.amount = new_amount;

        Ok(Idempotent::Executed(SendCollateralToLiquidationResult {
            ledger_tx_id,
            amount,
            collateral_account_id: self.account_id,
            collateral_in_liquidation_account_id: liquidation.collateral_in_liquidation_account_id,
            collateral_update: CollateralUpdate {
                tx_id: ledger_tx_id,
                abs_diff: amount,
                action: CollateralAction::Remove,
                effective,
            },
        }))
    }

    /// Initiates a new liquidation for this collateral.
    /// Returns error if there's already an active liquidation or no collateral to liquidate.
    pub fn initiate_liquidation(
        &mut self,
        new_liquidation: NewLiquidation,
        trigger_price: PriceOfOneBTC,
        initially_expected_to_receive: UsdCents,
        initially_estimated_to_liquidate: Satoshis,
    ) -> Result<Idempotent<()>, CollateralError> {
        if self.amount == Satoshis::ZERO {
            return Err(CollateralError::NoCollateralToLiquidate);
        }

        if self.has_active_liquidation() {
            return Err(CollateralError::LiquidationAlreadyActive);
        }

        let liquidation_id = new_liquidation.id;

        self.liquidations.add_new(new_liquidation);

        self.events.push(CollateralEvent::LiquidationStarted {
            liquidation_id,
            trigger_price,
            initially_expected_to_receive,
            initially_estimated_to_liquidate,
        });

        Ok(Idempotent::Executed(()))
    }

    /// Completes the liquidation with the given ID.
    pub fn complete_liquidation(
        &mut self,
        liquidation_id: LiquidationId,
    ) -> Result<Idempotent<()>, CollateralError> {
        let liquidation = self
            .liquidations
            .get_persisted_mut(&liquidation_id)
            .ok_or(CollateralError::LiquidationNotFound(liquidation_id))?;

        match liquidation.complete() {
            Idempotent::AlreadyApplied => return Ok(Idempotent::AlreadyApplied),
            Idempotent::Executed(()) => {}
        }

        self.events
            .push(CollateralEvent::LiquidationCompleted { liquidation_id });

        Ok(Idempotent::Executed(()))
    }

    /// Returns true if there's an active (uncompleted) liquidation.
    pub fn has_active_liquidation(&self) -> bool {
        self.active_liquidation_id().is_some()
    }

    /// Returns the ID of the active liquidation, if any.
    pub fn active_liquidation_id(&self) -> Option<LiquidationId> {
        self.events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                CollateralEvent::LiquidationCompleted { .. } => Some(None),
                CollateralEvent::LiquidationStarted { liquidation_id, .. } => {
                    Some(Some(*liquidation_id))
                }
                _ => None,
            })
            .flatten()
    }

    /// Returns a reference to the active liquidation, if any.
    pub fn active_liquidation(&self) -> Option<&Liquidation> {
        self.active_liquidation_id().map(|id| {
            self.liquidations
                .get_persisted(&id)
                .expect("Active liquidation not found in nested entities")
        })
    }

    /// Returns a mutable reference to the active liquidation, if any.
    pub fn active_liquidation_mut(&mut self) -> Option<&mut Liquidation> {
        self.active_liquidation_id().map(|id| {
            self.liquidations
                .get_persisted_mut(&id)
                .expect("Active liquidation not found in nested entities")
        })
    }

    /// Returns a reference to a liquidation by ID.
    pub fn liquidation(&self, liquidation_id: &LiquidationId) -> Option<&Liquidation> {
        self.liquidations.get_persisted(liquidation_id)
    }

    /// Returns a mutable reference to a liquidation by ID.
    pub fn liquidation_mut(&mut self, liquidation_id: &LiquidationId) -> Option<&mut Liquidation> {
        self.liquidations.get_persisted_mut(liquidation_id)
    }

    /// Records proceeds received from liquidation through the nested Liquidation entity.
    /// Returns the data needed for ledger posting.
    pub fn record_liquidation_proceeds_received(
        &mut self,
        amount_received: UsdCents,
        payment_id: PaymentId,
        ledger_tx_id: LedgerTxId,
    ) -> Result<Idempotent<RecordProceedsFromLiquidationData>, CollateralError> {
        let liquidation = self
            .active_liquidation_mut()
            .ok_or(CollateralError::NoActiveLiquidation)?;

        Ok(liquidation.record_proceeds_from_liquidation(
            amount_received,
            payment_id,
            ledger_tx_id,
        )?)
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
    pub(super) account_ids: CollateralLedgerAccountIds,
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
                    account_ids,
                } => {
                    builder = builder
                        .id(*id)
                        .account_id(*account_id)
                        .amount(Satoshis::ZERO)
                        .custody_wallet_id(*custody_wallet_id)
                        .credit_facility_id(*credit_facility_id)
                        .pending_credit_facility_id(*pending_credit_facility_id)
                        .account_ids(*account_ids)
                }
                CollateralEvent::UpdatedViaManualInput {
                    collateral_amount: new_value,
                    ..
                }
                | CollateralEvent::UpdatedViaCustodianSync {
                    collateral_amount: new_value,
                    ..
                }
                | CollateralEvent::UpdatedViaLiquidation {
                    collateral_amount: new_value,
                    ..
                } => {
                    builder = builder.amount(*new_value);
                }
                CollateralEvent::LiquidationStarted { .. } => {}
                CollateralEvent::LiquidationCompleted { .. } => {}
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
                account_ids: self.account_ids,
                custody_wallet_id: self.custody_wallet_id,
            }],
        )
    }
}
