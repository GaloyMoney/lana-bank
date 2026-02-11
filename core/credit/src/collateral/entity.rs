use chrono::{DateTime, Utc};
use derive_builder::Builder;
use money::UsdCents;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use es_entity::*;

use crate::primitives::{
    CalaAccountId, CollateralDirection, CollateralId, CreditFacilityId, CustodyWalletId,
    LedgerTxId, LiquidationId, PaymentId, PendingCreditFacilityId, PriceOfOneBTC, Satoshis,
};

use super::{
    CollateralUpdate, RecordProceedsFromLiquidationData,
    error::CollateralError,
    ledger::{CollateralLedgerAccountIds, LiquidationProceedsAccountIds},
    liquidation::{Liquidation, NewLiquidation},
};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "CollateralId")]
pub enum CollateralEvent {
    Initialized {
        id: CollateralId,
        credit_facility_id: CreditFacilityId,
        account_ids: CollateralLedgerAccountIds,
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
        ledger_tx_id: LedgerTxId,
        liquidation_id: LiquidationId,
        collateral_amount: Satoshis,
        abs_diff: Satoshis,
        direction: CollateralDirection,
    },
    LiquidationStarted {
        liquidation_id: LiquidationId,
    },
    LiquidationProceedsReceived {
        ledger_tx_id: LedgerTxId,
        payment_id: PaymentId,
        liquidation_id: LiquidationId,
        amount: UsdCents,
    },
    LiquidationCompleted {
        liquidation_id: LiquidationId,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Collateral {
    pub id: CollateralId,
    pub credit_facility_id: CreditFacilityId,
    pub account_ids: CollateralLedgerAccountIds,
    pub pending_credit_facility_id: PendingCreditFacilityId,
    pub custody_wallet_id: Option<CustodyWalletId>,
    pub amount: Satoshis,

    #[es_entity(nested)]
    #[builder(default)]
    liquidations: Nested<Liquidation>,

    events: EntityEvents<CollateralEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct CollateralAdjustment {
    pub tx_id: LedgerTxId,
    pub abs_diff: Satoshis,
    pub direction: CollateralDirection,
}

impl Collateral {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn last_adjustment(&self) -> Option<CollateralAdjustment> {
        self.events.iter_all().rev().find_map(|event| match event {
            CollateralEvent::UpdatedViaManualInput {
                ledger_tx_id,
                abs_diff,
                direction,
                ..
            }
            | CollateralEvent::UpdatedViaCustodianSync {
                ledger_tx_id,
                abs_diff,
                direction,
                ..
            }
            | CollateralEvent::UpdatedViaLiquidation {
                ledger_tx_id,
                abs_diff,
                direction,
                ..
            } => Some(CollateralAdjustment {
                tx_id: *ledger_tx_id,
                abs_diff: *abs_diff,
                direction: *direction,
            }),
            _ => None,
        })
    }

    pub fn collateral_account_id(&self) -> CalaAccountId {
        self.account_ids.collateral_account_id
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
            collateral_account_id: self.collateral_account_id(),
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
            collateral_account_id: self.collateral_account_id(),
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
            ledger_tx_id: tx_id,
            direction: CollateralDirection::Remove,
        });
        self.amount = new_amount;

        Ok(Idempotent::Executed(CollateralUpdate {
            tx_id,
            collateral_account_id: self.collateral_account_id(),
            abs_diff: amount_sent,
            direction: CollateralDirection::Remove,
            effective,
        }))
    }

    pub fn record_proceeds_received_and_liquidation_completed(
        &mut self,
        amount_received: UsdCents,
    ) -> Result<Idempotent<RecordProceedsFromLiquidationData>, CollateralError> {
        let (data, liquidation_id, sent_total, account_ids) = {
            let liquidation = self
                .active_liquidation()
                .ok_or(CollateralError::NoActiveLiquidation)?;

            let data = liquidation
                .record_proceeds_from_liquidation_and_complete(amount_received)
                .expect("Active liquidation in incorrect \"completed\" state");
            (
                data,
                liquidation.id,
                liquidation.sent_total,
                liquidation.account_ids,
            )
        };

        self.events
            .push(CollateralEvent::LiquidationProceedsReceived {
                liquidation_id,
                amount: amount_received,
                ledger_tx_id: data.ledger_tx_id,
                payment_id: data.payment_id,
            });

        self.events
            .push(CollateralEvent::LiquidationCompleted { liquidation_id });

        Ok(Idempotent::Executed(
            RecordProceedsFromLiquidationData::new(
                account_ids,
                amount_received,
                sent_total,
                data.ledger_tx_id,
            ),
        ))
    }

    fn active_liquidation_id(&self) -> Option<LiquidationId> {
        let mut active: Option<LiquidationId> = None;

        for event in self.events.iter_all() {
            match event {
                CollateralEvent::LiquidationStarted { liquidation_id, .. } => {
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
        trigger_price: PriceOfOneBTC,
        initially_expected_to_receive: UsdCents,
        initially_estimated_to_liquidate: Satoshis,
        liquidation_proceeds_account_ids: LiquidationProceedsAccountIds,
    ) -> Idempotent<LiquidationId> {
        if self.active_liquidation_id().is_some() {
            return Idempotent::AlreadyApplied;
        }

        let liquidation_id = LiquidationId::new();
        let new_liquidation = NewLiquidation::builder()
            .id(liquidation_id)
            .collateral_id(self.id)
            .trigger_price(trigger_price)
            .initially_expected_to_receive(initially_expected_to_receive)
            .initially_estimated_to_liquidate(initially_estimated_to_liquidate)
            .account_ids(liquidation_proceeds_account_ids)
            .build()
            .expect("all fields for new liquidation provided");

        self.liquidations.add_new(new_liquidation);

        self.events
            .push(CollateralEvent::LiquidationStarted { liquidation_id });

        Idempotent::Executed(liquidation_id)
    }
}

#[derive(Debug, Builder)]
pub struct NewCollateral {
    #[builder(setter(into))]
    pub(super) id: CollateralId,
    #[builder(setter(into))]
    pub(super) credit_facility_id: CreditFacilityId,
    #[builder(setter(into))]
    pub(super) pending_credit_facility_id: PendingCreditFacilityId,
    #[builder(default)]
    pub(super) custody_wallet_id: Option<CustodyWalletId>,
    pub(super) account_ids: CollateralLedgerAccountIds,
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
                    account_ids,
                } => {
                    builder = builder
                        .id(*id)
                        .account_ids(*account_ids)
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
                CollateralEvent::LiquidationProceedsReceived { .. } => {}
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
                credit_facility_id: self.credit_facility_id,
                account_ids: self.account_ids,
                pending_credit_facility_id: self.pending_credit_facility_id,
                custody_wallet_id: self.custody_wallet_id,
            }],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ledger::FacilityProceedsFromLiquidationAccountId,
        primitives::{PriceOfOneBTC, UsdCents},
    };

    fn default_account_ids() -> CollateralLedgerAccountIds {
        CollateralLedgerAccountIds::new()
    }

    fn default_liquidation_proceeds_account_ids() -> LiquidationProceedsAccountIds {
        LiquidationProceedsAccountIds {
            liquidation_proceeds_omnibus_account_id: CalaAccountId::new(),
            proceeds_from_liquidation_account_id: FacilityProceedsFromLiquidationAccountId::new(),
            collateral_in_liquidation_account_id: CalaAccountId::new(),
            liquidated_collateral_account_id: CalaAccountId::new(),
        }
    }

    fn default_new_collateral() -> NewCollateral {
        NewCollateral::builder()
            .id(CollateralId::new())
            .account_ids(default_account_ids())
            .credit_facility_id(CreditFacilityId::new())
            .pending_credit_facility_id(PendingCreditFacilityId::new())
            .build()
            .unwrap()
    }

    fn collateral_from(new_collateral: NewCollateral) -> Collateral {
        Collateral::try_from_events(new_collateral.into_events()).unwrap()
    }

    fn default_trigger_price() -> PriceOfOneBTC {
        PriceOfOneBTC::new(UsdCents::from(5000000))
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

    fn start_liquidation(collateral: &mut Collateral) -> LiquidationId {
        let result = collateral.record_liquidation_started(
            default_trigger_price(),
            UsdCents::from(1000),
            Satoshis::from(100000),
            default_liquidation_proceeds_account_ids(),
        );
        let liquidation_id = result.unwrap();
        hydrate_liquidations_in_collateral(collateral);
        liquidation_id
    }

    mod record_liquidation_started {
        use super::*;

        #[test]
        fn creates_liquidation() {
            let new_collateral = default_new_collateral();
            let mut collateral = collateral_from(new_collateral);

            let result = collateral.record_liquidation_started(
                default_trigger_price(),
                UsdCents::from(1000),
                Satoshis::from(100000),
                default_liquidation_proceeds_account_ids(),
            );
            assert!(result.did_execute());
            let liquidation_id = result.unwrap();

            hydrate_liquidations_in_collateral(&mut collateral);
            assert_eq!(collateral.active_liquidation_id(), Some(liquidation_id));
        }

        #[test]
        fn returns_already_applied_if_active_liquidation_exists() {
            let new_collateral = default_new_collateral();
            let mut collateral = collateral_from(new_collateral);

            start_liquidation(&mut collateral);

            let result = collateral.record_liquidation_started(
                default_trigger_price(),
                UsdCents::from(1000),
                Satoshis::from(100000),
                default_liquidation_proceeds_account_ids(),
            );
            assert!(result.was_already_applied());
        }
    }

    mod record_collateral_update_via_liquidation {
        use super::*;

        #[test]
        fn updates_via_active_liquidation() {
            let new_collateral = default_new_collateral();
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
            start_liquidation(&mut collateral);

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
            let mut collateral = collateral_from(new_collateral);

            start_liquidation(&mut collateral);

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

    mod record_proceeds_received_and_liquidation_completed {
        use super::*;

        #[test]
        fn records_proceeds_and_completes_liquidation() {
            let new_collateral = default_new_collateral();
            let mut collateral = collateral_from(new_collateral);

            start_liquidation(&mut collateral);

            let amount = UsdCents::from(500);
            let result = collateral.record_proceeds_received_and_liquidation_completed(amount);
            assert!(result.is_ok());
            assert!(result.unwrap().did_execute());
            assert!(collateral.active_liquidation_id().is_none());
        }

        #[test]
        fn fails_if_no_active_liquidation() {
            let new_collateral = default_new_collateral();
            let mut collateral = collateral_from(new_collateral);

            let result =
                collateral.record_proceeds_received_and_liquidation_completed(UsdCents::from(500));
            assert!(matches!(result, Err(CollateralError::NoActiveLiquidation)));
        }
    }
}
