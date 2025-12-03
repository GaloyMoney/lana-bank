use derive_builder::Builder;
use es_entity::*;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::primitives::{DomainConfigId, DomainConfigKey};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "DomainConfigId")]
pub enum DomainConfigEvent {
    Initialized {
        id: DomainConfigId,
        key: DomainConfigKey,
        value: serde_json::Value,
    },
    Updated {
        value: serde_json::Value,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct DomainConfig {
    pub id: DomainConfigId,
    pub key: DomainConfigKey,
    events: EntityEvents<DomainConfigEvent>,
}

impl DomainConfig {
    pub(super) fn apply_update(&mut self, new_value: serde_json::Value) {
        let event = DomainConfigEvent::Updated {
            value: new_value.clone(),
        };

        self.events.push(event);
    }

    pub(super) fn current_value<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        let last_event = self
            .events
            .iter_all()
            .next_back()
            .expect("last event exists");
        let value = match last_event {
            DomainConfigEvent::Initialized { value, .. } => value,
            DomainConfigEvent::Updated { value } => value,
        };
        serde_json::from_value(value.clone())
    }
}

impl TryFromEvents<DomainConfigEvent> for DomainConfig {
    fn try_from_events(events: EntityEvents<DomainConfigEvent>) -> Result<Self, EsEntityError> {
        let mut builder = DomainConfigBuilder::default();

        for event in events.iter_all() {
            match event {
                DomainConfigEvent::Initialized { id, key, .. } => {
                    builder = builder.id(*id).key(key.clone());
                }
                DomainConfigEvent::Updated { .. } => {}
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewDomainConfig {
    pub(super) id: DomainConfigId,
    pub(super) key: DomainConfigKey,
    value: serde_json::Value,
}

impl NewDomainConfig {
    pub fn builder() -> NewDomainConfigBuilder {
        NewDomainConfigBuilder::default()
    }
}

impl IntoEvents<DomainConfigEvent> for NewDomainConfig {
    fn into_events(self) -> EntityEvents<DomainConfigEvent> {
        EntityEvents::init(
            self.id,
            [DomainConfigEvent::Initialized {
                id: self.id,
                key: self.key,
                value: self.value.clone(),
            }],
        )
    }
}
