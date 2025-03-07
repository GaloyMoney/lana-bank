use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::DepositConfigId;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "DepositConfigId")]
pub enum DepositConfigEvent {
    Initialized { id: DepositConfigId },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct DepositConfig {
    pub id: DepositConfigId,
    pub(super) events: EntityEvents<DepositConfigEvent>,
}

impl TryFromEvents<DepositConfigEvent> for DepositConfig {
    fn try_from_events(events: EntityEvents<DepositConfigEvent>) -> Result<Self, EsEntityError> {
        let mut builder = DepositConfigBuilder::default();
        for event in events.iter_all() {
            match event {
                DepositConfigEvent::Initialized { id } => builder = builder.id(*id),
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewDepositConfig {
    #[builder(setter(into))]
    pub(super) id: DepositConfigId,
}

impl NewDepositConfig {
    pub fn builder() -> NewDepositConfigBuilder {
        NewDepositConfigBuilder::default()
    }
}

impl IntoEvents<DepositConfigEvent> for NewDepositConfig {
    fn into_events(self) -> EntityEvents<DepositConfigEvent> {
        EntityEvents::init(self.id, [DepositConfigEvent::Initialized { id: self.id }])
    }
}
