use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{entity::*, primitives::*};

use super::{error::*, state::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FixedTermLoanEvent {
    Initialized {
        id: FixedTermLoanId,
        state: FixedTermLoanState,
    },
    LedgerAccountCreated {
        ledger_account_id: LedgerAccountId,
        state: FixedTermLoanState,
    },
    CollateralizationAdded {
        state: FixedTermLoanState,
    },
}

impl EntityEvent for FixedTermLoanEvent {
    type EntityId = FixedTermLoanId;
    fn event_table_name() -> &'static str {
        "fixed_term_loan_events"
    }
}

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct FixedTermLoan {
    pub id: FixedTermLoanId,
    pub state: FixedTermLoanState,
    pub(super) events: EntityEvents<FixedTermLoanEvent>,
}

impl FixedTermLoan {
    pub fn set_ledger_account_id(
        &mut self,
        ledger_account_id: LedgerAccountId,
    ) -> Result<(), FixedTermLoanError> {
        if self.state != FixedTermLoanState::Initializing {
            return Err(FixedTermLoanError::BadState(
                FixedTermLoanState::Initializing,
                self.state,
            ));
        }
        self.events.push(FixedTermLoanEvent::LedgerAccountCreated {
            ledger_account_id,
            state: FixedTermLoanState::PendingCollateralization,
        });
        Ok(())
    }

    pub fn declare_collateralized(&mut self) -> Result<(), FixedTermLoanError> {
        if self.state != FixedTermLoanState::Collateralized {
            return Err(FixedTermLoanError::BadState(
                FixedTermLoanState::Collateralized,
                self.state,
            ));
        }
        self.events
            .push(FixedTermLoanEvent::CollateralizationAdded {
                state: FixedTermLoanState::Collateralized,
            });
        Ok(())
    }
}

impl Entity for FixedTermLoan {
    type Event = FixedTermLoanEvent;
}

impl TryFrom<EntityEvents<FixedTermLoanEvent>> for FixedTermLoan {
    type Error = EntityError;

    fn try_from(events: EntityEvents<FixedTermLoanEvent>) -> Result<Self, Self::Error> {
        let mut builder = FixedTermLoanBuilder::default();
        for event in events.iter() {
            match event {
                FixedTermLoanEvent::Initialized { id, state } => {
                    builder = builder.id(*id).state(*state);
                }
                FixedTermLoanEvent::LedgerAccountCreated { state, .. } => {
                    builder = builder.state(*state);
                }
                FixedTermLoanEvent::CollateralizationAdded { state } => {
                    builder = builder.state(*state);
                }
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewFixedTermLoan {
    #[builder(setter(into))]
    pub(super) id: FixedTermLoanId,
}

impl NewFixedTermLoan {
    pub fn builder() -> NewFixedTermLoanBuilder {
        NewFixedTermLoanBuilder::default()
    }

    pub(super) fn initial_events(self) -> EntityEvents<FixedTermLoanEvent> {
        EntityEvents::init(
            self.id,
            [FixedTermLoanEvent::Initialized {
                id: self.id,
                state: FixedTermLoanState::Initializing,
            }],
        )
    }
}
