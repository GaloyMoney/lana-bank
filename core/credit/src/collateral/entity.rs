use chrono::{DateTime, Utc};
use core_money::UsdCents;
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
    liquidation::{Liquidation, NewLiquidation, RecordProceedsFromLiquidationData},
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
    ) -> Result<Idempotent<CollateralUpdate>, CollateralError> {
        if self.custody_wallet_id.is_some() {
            return Err(CollateralError::ManualUpdateError);
        }

        let current = self.amount;

        let (abs_diff, direction) = match new_amount.cmp(&current) {
            Ordering::Less => (current - new_amount, CollateralDirection::Remove),
            Ordering::Greater => (new_amount - current, CollateralDirection::Add),
            Ordering::Equal => return Ok(Idempotent::AlreadyApplied),
        };

        let tx_id = LedgerTxId::new();

        self.events.push(CollateralEvent::UpdatedViaManualInput {
            ledger_tx_id: tx_id,
            abs_diff,
            collateral_amount: new_amount,
            direction,
        });

        self.amount = new_amount;

        Ok(Idempotent::Executed(CollateralUpdate {
            tx_id,
            abs_diff,
            direction,
            effective,
        }))
    }

    pub fn record_collateral_update_via_liquidation(
        &mut self,
        amount_sent: Satoshis,
        effective: chrono::NaiveDate,
    ) -> Result<Idempotent<CollateralUpdate>, CollateralError> {
        if amount_sent == Satoshis::ZERO {
            return Ok(Idempotent::AlreadyApplied);
        }

        let new_amount = self.amount - amount_sent;

        let (liquidation_id, tx_id) = {
            let liquidation = self
                .active_liquidation()
                .ok_or(CollateralError::NoActiveLiquidation)?;

            let tx_id = if let Idempotent::Executed(tx_id) =
                liquidation.record_collateral_sent_out(amount_sent)
            {
                tx_id
            } else {
                return Ok(Idempotent::AlreadyApplied);
            };

            (liquidation.id, tx_id)
        };

        self.events.push(CollateralEvent::UpdatedViaLiquidation {
            liquidation_id,
            abs_diff: amount_sent,
            collateral_amount: new_amount,
            direction: CollateralDirection::Remove,
        });
        self.amount = new_amount;

        Ok(Idempotent::Executed(CollateralUpdate {
            tx_id,
            abs_diff: amount_sent,
            direction: CollateralDirection::Remove,
            effective,
        }))
    }

    pub fn record_liquidation_proceeds_received(
        &mut self,
        amount_received: UsdCents,
    ) -> Result<Idempotent<RecordProceedsFromLiquidationData>, CollateralError> {
        let liquidation = self
            .active_liquidation()
            .ok_or(CollateralError::NoActiveLiquidation)?;

        Ok(liquidation.record_proceeds_from_liquidation(amount_received)?)
    }

    pub(super) fn collateral_in_liquidation_account_id(&self) -> Option<CalaAccountId> {
        self.events
            .iter_all()
            .find_map(|e| match e {
                CollateralEvent::LiquidationStarted { liquidation_id } => Some(*liquidation_id),
                _ => None,
            })
            .and_then(|liquidation_id| {
                self.liquidations
                    .get_persisted(&liquidation_id)
                    .map(|l| l.collateral_in_liquidation_account_id)
            })
    }

    fn active_liquidation_id(&self) -> Option<LiquidationId> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::FacilityProceedsFromLiquidationAccountId;
    use crate::primitives::{PriceOfOneBTC, UsdCents};

    fn default_new_collateral() -> NewCollateral {
        NewCollateral::builder()
            .id(CollateralId::new())
            .account_id(CalaAccountId::new())
            .credit_facility_id(CreditFacilityId::new())
            .pending_credit_facility_id(PendingCreditFacilityId::new())
            .build()
            .unwrap()
    }

    fn collateral_from(new_collateral: NewCollateral) -> Collateral {
        Collateral::try_from_events(new_collateral.into_events()).unwrap()
    }

    fn default_new_liquidation(collateral_id: CollateralId) -> NewLiquidation {
        NewLiquidation::builder()
            .id(LiquidationId::new())
            .credit_facility_id(CreditFacilityId::new())
            .collateral_id(collateral_id)
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

    fn hydrate_liquidations_in_collateral(collateral: &mut Collateral) {
        let new_entities = collateral
            .liquidations
            .new_entities_mut()
            .drain(..)
            .map(|new| Liquidation::try_from_events(new.into_events()).expect("hydrate failed"))
            .collect::<Vec<_>>();
        collateral.liquidations.load(new_entities);
    }

    mod record_liquidation_started {
        use super::*;

        #[test]
        fn creates_liquidation() {
            let new_collateral = default_new_collateral();
            let collateral_id = new_collateral.id;
            let mut collateral = collateral_from(new_collateral);

            let new_liquidation = default_new_liquidation(collateral_id);
            let liquidation_id = new_liquidation.id;
            let result = collateral.record_liquidation_started(new_liquidation);
            assert!(result.is_ok());
            assert!(result.unwrap().did_execute());

            hydrate_liquidations_in_collateral(&mut collateral);
            assert_eq!(collateral.active_liquidation_id(), Some(liquidation_id));
        }

        #[test]
        fn is_idempotent() {
            let new_collateral = default_new_collateral();
            let collateral_id = new_collateral.id;
            let mut collateral = collateral_from(new_collateral);

            let new_liquidation = default_new_liquidation(collateral_id);
            let liquidation_id = new_liquidation.id;
            let _ = collateral
                .record_liquidation_started(new_liquidation)
                .unwrap();
            hydrate_liquidations_in_collateral(&mut collateral);

            let duplicate_liquidation = NewLiquidation::builder()
                .id(liquidation_id)
                .credit_facility_id(CreditFacilityId::new())
                .collateral_id(collateral_id)
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
                .unwrap();

            let result = collateral.record_liquidation_started(duplicate_liquidation);
            assert!(result.is_ok());
            assert!(result.unwrap().was_already_applied());
        }

        #[test]
        fn fails_if_active_liquidation_exists() {
            let new_collateral = default_new_collateral();
            let collateral_id = new_collateral.id;
            let mut collateral = collateral_from(new_collateral);

            let new_liquidation = default_new_liquidation(collateral_id);
            let _ = collateral
                .record_liquidation_started(new_liquidation)
                .unwrap();
            hydrate_liquidations_in_collateral(&mut collateral);

            let another_liquidation = default_new_liquidation(collateral_id);
            let result = collateral.record_liquidation_started(another_liquidation);
            assert!(matches!(
                result,
                Err(CollateralError::ActiveLiquidationExists(_))
            ));
        }
    }

    mod record_liquidation_completed {
        use super::*;

        #[test]
        fn completes_liquidation() {
            let new_collateral = default_new_collateral();
            let collateral_id = new_collateral.id;
            let mut collateral = collateral_from(new_collateral);

            let new_liquidation = default_new_liquidation(collateral_id);
            let liquidation_id = new_liquidation.id;
            let _ = collateral
                .record_liquidation_started(new_liquidation)
                .unwrap();
            hydrate_liquidations_in_collateral(&mut collateral);

            let result = collateral.record_liquidation_completed(liquidation_id);
            assert!(result.is_ok());
            assert!(result.unwrap().did_execute());
            assert!(collateral.active_liquidation_id().is_none());
        }

        #[test]
        fn is_idempotent() {
            let new_collateral = default_new_collateral();
            let collateral_id = new_collateral.id;
            let mut collateral = collateral_from(new_collateral);

            let new_liquidation = default_new_liquidation(collateral_id);
            let liquidation_id = new_liquidation.id;
            let _ = collateral
                .record_liquidation_started(new_liquidation)
                .unwrap();
            hydrate_liquidations_in_collateral(&mut collateral);

            let _ = collateral
                .record_liquidation_completed(liquidation_id)
                .unwrap();
            let result = collateral.record_liquidation_completed(liquidation_id);
            assert!(result.is_ok());
            assert!(result.unwrap().was_already_applied());
        }

        #[test]
        fn fails_if_no_active_liquidation() {
            let new_collateral = default_new_collateral();
            let mut collateral = collateral_from(new_collateral);

            let result = collateral.record_liquidation_completed(LiquidationId::new());
            assert!(matches!(result, Err(CollateralError::NoActiveLiquidation)));
        }
    }

    mod record_collateral_update_via_liquidation {
        use super::*;

        #[test]
        fn updates_via_active_liquidation() {
            let new_collateral = default_new_collateral();
            let collateral_id = new_collateral.id;
            let mut collateral = collateral_from(new_collateral);

            // First add some collateral
            let initial_amount = Satoshis::from(100000);
            let _ = collateral
                .record_collateral_update_via_manual_input(
                    initial_amount,
                    chrono::Utc::now().date_naive(),
                )
                .unwrap();

            // Start a liquidation
            let new_liquidation = default_new_liquidation(collateral_id);
            let _ = collateral
                .record_liquidation_started(new_liquidation)
                .unwrap();
            hydrate_liquidations_in_collateral(&mut collateral);

            // Update collateral via liquidation
            let amount_to_remove = Satoshis::from(50000);
            let result = collateral.record_collateral_update_via_liquidation(
                amount_to_remove,
                chrono::Utc::now().date_naive(),
            );
            assert!(result.is_ok());
            assert!(result.unwrap().did_execute());
            assert_eq!(collateral.amount, initial_amount - amount_to_remove);
        }

        #[test]
        fn returns_already_applied_for_zero_amount() {
            let new_collateral = default_new_collateral();
            let collateral_id = new_collateral.id;
            let mut collateral = collateral_from(new_collateral);

            let new_liquidation = default_new_liquidation(collateral_id);
            let _ = collateral
                .record_liquidation_started(new_liquidation)
                .unwrap();
            hydrate_liquidations_in_collateral(&mut collateral);

            let result = collateral.record_collateral_update_via_liquidation(
                Satoshis::ZERO,
                chrono::Utc::now().date_naive(),
            );
            assert!(result.is_ok());
            assert!(result.unwrap().was_already_applied());
        }

        #[test]
        fn fails_if_no_active_liquidation() {
            let new_collateral = default_new_collateral();
            let mut collateral = collateral_from(new_collateral);

            // Add some collateral first to avoid arithmetic overflow
            let _ = collateral
                .record_collateral_update_via_manual_input(
                    Satoshis::from(100000),
                    chrono::Utc::now().date_naive(),
                )
                .unwrap();

            let result = collateral.record_collateral_update_via_liquidation(
                Satoshis::from(50000),
                chrono::Utc::now().date_naive(),
            );
            assert!(matches!(result, Err(CollateralError::NoActiveLiquidation)));
        }
    }

    mod record_liquidation_proceeds_received {
        use super::*;

        #[test]
        fn records_proceeds() {
            let new_collateral = default_new_collateral();
            let collateral_id = new_collateral.id;
            let mut collateral = collateral_from(new_collateral);

            let new_liquidation = default_new_liquidation(collateral_id);
            let _ = collateral
                .record_liquidation_started(new_liquidation)
                .unwrap();
            hydrate_liquidations_in_collateral(&mut collateral);

            let amount = UsdCents::from(500);
            let result = collateral.record_liquidation_proceeds_received(amount);
            assert!(result.is_ok());
            assert!(result.unwrap().did_execute());
        }

        #[test]
        fn fails_if_no_active_liquidation() {
            let new_collateral = default_new_collateral();
            let mut collateral = collateral_from(new_collateral);

            let result = collateral.record_liquidation_proceeds_received(UsdCents::from(500));
            assert!(matches!(result, Err(CollateralError::NoActiveLiquidation)));
        }
    }
}
