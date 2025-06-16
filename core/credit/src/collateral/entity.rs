use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use audit::AuditInfo;
use es_entity::*;

use cala_ledger::AccountId as CalaAccountId;

use crate::primitives::{CollateralAction, CollateralId, CreditFacilityId, LedgerTxId, Satoshis};

use super::CollateralUpdate;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "CollateralId")]
pub enum CollateralEvent {
    Initialized {
        id: CollateralId,
        account_id: CalaAccountId,
        credit_facility_id: CreditFacilityId,
    },
    Updated {
        ledger_tx_id: LedgerTxId,
        collateral_amount: Satoshis,
        abs_diff: Satoshis,
        action: CollateralAction,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Collateral {
    pub id: CollateralId,
    pub credit_facility_id: CreditFacilityId,
    pub amount: Satoshis,

    events: EntityEvents<CollateralEvent>,
}

impl Collateral {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn record_collateral_update(
        &mut self,
        new_amount: Satoshis,
        effective: chrono::NaiveDate,
        audit_info: &AuditInfo,
    ) -> Idempotent<CollateralUpdate> {
        let current = self.amount;

        let (abs_diff, action) = match new_amount.cmp(&current) {
            Ordering::Less => (current - new_amount, CollateralAction::Remove),
            Ordering::Greater => (new_amount - current, CollateralAction::Add),
            Ordering::Equal => return Idempotent::Ignored,
        };

        let tx_id = LedgerTxId::new();

        self.events.push(CollateralEvent::Updated {
            ledger_tx_id: tx_id,
            abs_diff,
            collateral_amount: new_amount,
            action,
            audit_info: audit_info.clone(),
        });

        self.amount = new_amount;

        Idempotent::Executed(CollateralUpdate {
            tx_id,
            abs_diff,
            action,
            effective,
        })
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
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .amount(Satoshis::ZERO)
                        .credit_facility_id(*credit_facility_id);
                }
                CollateralEvent::Updated {
                    collateral_amount: new_value,
                    ..
                } => {
                    builder = builder.amount(*new_value);
                }
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
            }],
        )
    }
}
