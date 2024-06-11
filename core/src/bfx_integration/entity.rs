use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{
    entity::*,
    primitives::{BfxIntegrationId, LedgerAccountId, LedgerAccountSetId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BfxIntegrationEvent {
    Initialized {
        id: BfxIntegrationId,
        omnibus_account_set_id: LedgerAccountSetId,
        withdrawal_account_id: LedgerAccountId,
    },
}

impl EntityEvent for BfxIntegrationEvent {
    type EntityId = BfxIntegrationId;
    fn event_table_name() -> &'static str {
        "bfx_integration_events"
    }
}

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct BfxIntegration {
    pub id: BfxIntegrationId,
    pub omnibus_account_set_id: LedgerAccountSetId,
    pub withdrawal_account_id: LedgerAccountId,
    pub(super) events: EntityEvents<BfxIntegrationEvent>,
}

impl Entity for BfxIntegration {
    type Event = BfxIntegrationEvent;
}

impl TryFrom<EntityEvents<BfxIntegrationEvent>> for BfxIntegration {
    type Error = EntityError;

    fn try_from(events: EntityEvents<BfxIntegrationEvent>) -> Result<Self, Self::Error> {
        let mut builder = BfxIntegrationBuilder::default();
        for event in events.iter() {
            match event {
                BfxIntegrationEvent::Initialized {
                    id,
                    omnibus_account_set_id,
                    withdrawal_account_id,
                } => {
                    builder = builder
                        .id(*id)
                        .omnibus_account_set_id(*omnibus_account_set_id)
                        .withdrawal_account_id(*withdrawal_account_id);
                }
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewBfxIntegration {
    #[builder(setter(into))]
    pub(super) id: BfxIntegrationId,
    #[builder(setter(into))]
    pub(super) omnibus_account_set_id: LedgerAccountSetId,
    #[builder(setter(into))]
    pub(super) withdrawal_account_id: LedgerAccountId,
}

impl NewBfxIntegration {
    pub fn builder() -> NewBfxIntegrationBuilder {
        NewBfxIntegrationBuilder::default()
    }

    pub(super) fn initial_events(self) -> EntityEvents<BfxIntegrationEvent> {
        EntityEvents::init(
            self.id,
            [BfxIntegrationEvent::Initialized {
                id: self.id,
                omnibus_account_set_id: self.omnibus_account_set_id,
                withdrawal_account_id: self.withdrawal_account_id,
            }],
        )
    }
}
