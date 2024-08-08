use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{
    entity::*,
    primitives::{CustomerId, DepositId, LedgerAccountId, UsdCents},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DepositEvent {
    Initialized {
        id: DepositId,
        customer_id: CustomerId,
        amount: UsdCents,
        reference: String,
        credit_account_id: LedgerAccountId,
    },
}

impl EntityEvent for DepositEvent {
    type EntityId = DepositId;
    fn event_table_name() -> &'static str {
        "withdraw_events"
    }
}

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct Deposit {
    pub id: DepositId,
    pub customer_id: CustomerId,
    pub amount: UsdCents,
    pub credit_account_id: LedgerAccountId,
    pub(super) events: EntityEvents<DepositEvent>,
}

impl std::fmt::Display for Deposit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Deposit {}, uid: {}", self.id, self.customer_id)
    }
}

impl Entity for Deposit {
    type Event = DepositEvent;
}

impl TryFrom<EntityEvents<DepositEvent>> for Deposit {
    type Error = EntityError;

    fn try_from(events: EntityEvents<DepositEvent>) -> Result<Self, Self::Error> {
        let mut builder = DepositBuilder::default();
        for event in events.iter() {
            match event {
                DepositEvent::Initialized {
                    id,
                    customer_id,
                    amount,
                    credit_account_id,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .customer_id(*customer_id)
                        .amount(*amount)
                        .credit_account_id(*credit_account_id);
                }
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewDeposit {
    #[builder(setter(into))]
    pub(super) id: DepositId,
    #[builder(setter(into))]
    pub(super) customer_id: CustomerId,
    #[builder(setter(into))]
    pub(super) amount: UsdCents,
    reference: Option<String>,
    pub(super) credit_account_id: LedgerAccountId,
}

impl NewDeposit {
    pub fn builder() -> NewDepositBuilder {
        NewDepositBuilder::default()
    }

    pub(super) fn reference(&self) -> String {
        self.reference
            .clone()
            .unwrap_or_else(|| self.id.to_string())
    }

    pub(super) fn initial_events(self) -> EntityEvents<DepositEvent> {
        EntityEvents::init(
            self.id,
            [DepositEvent::Initialized {
                reference: self.reference(),
                id: self.id,
                customer_id: self.customer_id,
                amount: self.amount,
                credit_account_id: self.credit_account_id,
            }],
        )
    }
}
