use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{
    entity::*,
    primitives::{LedgerAccountId, UsdCents, UserId, WithdrawId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WithdrawEvent {
    Initialized {
        id: WithdrawId,
        user_id: UserId,
        amount: UsdCents,
        reference: String,
        destination: String,
        debit_account_id: LedgerAccountId,
    },
}

impl EntityEvent for WithdrawEvent {
    type EntityId = WithdrawId;
    fn event_table_name() -> &'static str {
        "withdraw_events"
    }
}

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct Withdraw {
    pub id: WithdrawId,
    pub user_id: UserId,
    pub amount: UsdCents,
    pub destination: String,
    pub debit_account_id: LedgerAccountId,
    pub(super) events: EntityEvents<WithdrawEvent>,
}

impl Entity for Withdraw {
    type Event = WithdrawEvent;
}

impl TryFrom<EntityEvents<WithdrawEvent>> for Withdraw {
    type Error = EntityError;

    fn try_from(events: EntityEvents<WithdrawEvent>) -> Result<Self, Self::Error> {
        let mut builder = WithdrawBuilder::default();
        for event in events.iter() {
            match event {
                WithdrawEvent::Initialized {
                    id,
                    user_id,
                    amount,
                    destination,
                    debit_account_id,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .user_id(*user_id)
                        .amount(*amount)
                        .destination(destination.clone())
                        .debit_account_id(*debit_account_id);
                }
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewWithdraw {
    #[builder(setter(into))]
    pub(super) id: WithdrawId,
    #[builder(setter(into))]
    pub(super) user_id: UserId,
    #[builder(setter(into))]
    pub(super) amount: UsdCents,
    reference: Option<String>,
    pub(super) destination: String,
    pub(super) debit_account_id: LedgerAccountId,
}

impl NewWithdraw {
    pub fn builder() -> NewWithdrawBuilder {
        NewWithdrawBuilder::default()
    }

    pub(super) fn reference(&self) -> String {
        self.reference
            .clone()
            .unwrap_or_else(|| self.id.to_string())
    }

    pub(super) fn initial_events(self) -> EntityEvents<WithdrawEvent> {
        EntityEvents::init(
            self.id,
            [WithdrawEvent::Initialized {
                reference: self.reference(),
                id: self.id,
                user_id: self.user_id,
                amount: self.amount,
                destination: self.destination,
                debit_account_id: self.debit_account_id,
            }],
        )
    }
}
