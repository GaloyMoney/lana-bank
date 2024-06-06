use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{
    entity::*,
    ledger::bank::BankLedgerAccountIds,
    primitives::{BankId, LedgerTxId},
};

use super::error::BankError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BankEvent {
    Initialized {
        id: BankId,
        account_ids: BankLedgerAccountIds,
    },
    EquityAdded {
        tx_id: LedgerTxId,
        reference: String,
    },
}

impl EntityEvent for BankEvent {
    type EntityId = BankId;
    fn event_table_name() -> &'static str {
        "bank_events"
    }
}

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct Bank {
    pub id: BankId,
    pub account_ids: BankLedgerAccountIds,
    pub(super) events: EntityEvents<BankEvent>,
}

impl Bank {
    pub fn add_equity(&mut self, tx_id: LedgerTxId, reference: String) -> Result<(), BankError> {
        self.events
            .push(BankEvent::EquityAdded { tx_id, reference });
        Ok(())
    }
}

impl Entity for Bank {
    type Event = BankEvent;
}

impl TryFrom<EntityEvents<BankEvent>> for Bank {
    type Error = EntityError;

    fn try_from(events: EntityEvents<BankEvent>) -> Result<Self, Self::Error> {
        let mut builder = BankBuilder::default();
        for event in events.iter() {
            match event {
                BankEvent::Initialized { id, account_ids } => {
                    builder = builder.id(*id).account_ids(*account_ids);
                }
                BankEvent::EquityAdded { .. } => {}
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewBank {
    #[builder(setter(into))]
    pub(super) id: BankId,
    #[builder(setter(into))]
    pub(super) account_ids: BankLedgerAccountIds,
}

impl NewBank {
    pub fn builder() -> NewBankBuilder {
        NewBankBuilder::default()
    }

    pub(super) fn initial_events(self) -> EntityEvents<BankEvent> {
        EntityEvents::init(
            self.id,
            [BankEvent::Initialized {
                id: self.id,
                account_ids: self.account_ids,
            }],
        )
    }
}
