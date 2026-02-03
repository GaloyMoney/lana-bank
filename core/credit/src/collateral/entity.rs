use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use es_entity::*;

use crate::primitives::{
    CalaAccountId, CollateralDirection, CollateralId, CreditFacilityId, CustodyWalletId,
    LedgerTxId, LiquidationId, PendingCreditFacilityId, Satoshis,
};

use super::{
    CollateralUpdate,
    error::CollateralError,
    liquidation::{Liquidation, NewLiquidation},
};

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
        direction: CollateralDirection,
    },
    UpdatedViaCustodianSync {
        ledger_tx_id: LedgerTxId,
        collateral_amount: Satoshis,
        abs_diff: Satoshis,
        direction: CollateralDirection,
    },
    UpdatedViaLiquidation {
        liquidation_id: LiquidationId,
        collateral_amount: Satoshis,
        abs_diff: Satoshis,
        direction: CollateralDirection,
    },
    LiquidationStarted {
        liquidation_id: LiquidationId,
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

        let (abs_diff, direction) = match new_amount.cmp(&current) {
            Ordering::Less => (current - new_amount, CollateralDirection::Remove),
            Ordering::Greater => (new_amount - current, CollateralDirection::Add),
            Ordering::Equal => return Idempotent::AlreadyApplied,
        };

        let tx_id = LedgerTxId::new();

        self.events.push(CollateralEvent::UpdatedViaCustodianSync {
            ledger_tx_id: tx_id,
            abs_diff,
            collateral_amount: new_amount,
            direction,
        });

        self.amount = new_amount;

        Idempotent::Executed(CollateralUpdate {
            tx_id,
            abs_diff,
            direction,
            effective,
        })
    }

    pub fn record_collateral_update_via_manual_input(
        &mut self,
        new_amount: Satoshis,
        effective: chrono::NaiveDate,
    ) -> Idempotent<CollateralUpdate> {
        let current = self.amount;

        let (abs_diff, direction) = match new_amount.cmp(&current) {
            Ordering::Less => (current - new_amount, CollateralDirection::Remove),
            Ordering::Greater => (new_amount - current, CollateralDirection::Add),
            Ordering::Equal => return Idempotent::AlreadyApplied,
        };

        let tx_id = LedgerTxId::new();

        self.events.push(CollateralEvent::UpdatedViaManualInput {
            ledger_tx_id: tx_id,
            abs_diff,
            collateral_amount: new_amount,
            direction,
        });

        self.amount = new_amount;

        Idempotent::Executed(CollateralUpdate {
            tx_id,
            abs_diff,
            direction,
            effective,
        })
    }

    pub fn record_collateral_update_via_liquidation(
        &mut self,
        liquidation_id: LiquidationId,
        amount_sent: Satoshis,
        effective: chrono::NaiveDate,
    ) -> Idempotent<CollateralUpdate> {
        if amount_sent == Satoshis::ZERO {
            return Idempotent::AlreadyApplied;
        }

        let new_amount = self.amount - amount_sent;

        let tx_id = LedgerTxId::new();

        self.events.push(CollateralEvent::UpdatedViaLiquidation {
            liquidation_id,
            abs_diff: amount_sent,
            collateral_amount: new_amount,
            direction: CollateralDirection::Remove,
        });

        self.amount = new_amount;

        Idempotent::Executed(CollateralUpdate {
            tx_id,
            abs_diff: amount_sent,
            direction: CollateralDirection::Remove,
            effective,
        })
    }

    pub fn active_liquidation_id(&self) -> Option<LiquidationId> {
        let mut active: Option<LiquidationId> = None;

        for event in self.events.iter_all() {
            match event {
                CollateralEvent::LiquidationStarted { liquidation_id } => {
                    active = Some(*liquidation_id);
                }
                CollateralEvent::LiquidationCompleted { liquidation_id } => {
                    if active == Some(*liquidation_id) {
                        active = None;
                    }
                }
                _ => {}
            }
        }

        active
    }

    fn active_liquidation(&mut self) -> Option<&mut Liquidation> {
        self.active_liquidation_id()
            .and_then(|id| self.liquidations.get_persisted_mut(&id))
    }

    pub fn record_liquidation_started(
        &mut self,
        new_liquidation @ NewLiquidation {
            id: liquidation_id, ..
        }: NewLiquidation,
    ) -> Result<Idempotent<LiquidationId>, CollateralError> {
        idempotency_guard!(
            self.events.iter_all(),
            CollateralEvent::LiquidationStarted { liquidation_id: id }
                if *id == liquidation_id,
        );

        if let Some(id) = self.active_liquidation_id() {
            return Err(CollateralError::ActiveLiquidationExists(id));
        }

        self.liquidations.add_new(new_liquidation);

        self.events
            .push(CollateralEvent::LiquidationStarted { liquidation_id });

        Ok(Idempotent::Executed(liquidation_id))
    }

    pub fn record_liquidation_completed(
        &mut self,
        liquidation_id: LiquidationId,
    ) -> Result<Idempotent<()>, CollateralError> {
        idempotency_guard!(
            self.events.iter_all(),
            CollateralEvent::LiquidationCompleted { liquidation_id: id }
                if *id == liquidation_id
        );

        let liquidation = match self.active_liquidation() {
            Some(l) => l,
            None => return Err(CollateralError::NoActiveLiquidation),
        };
        if liquidation.complete().was_already_applied() {
            return Ok(Idempotent::AlreadyApplied);
        }

        self.events
            .push(CollateralEvent::LiquidationCompleted { liquidation_id });

        Ok(Idempotent::Executed(()))
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
                custody_wallet_id: self.custody_wallet_id,
            }],
        )
    }
}
