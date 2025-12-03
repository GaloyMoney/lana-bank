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
    pub(super) fn update(&mut self, new_value: serde_json::Value) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            DomainConfigEvent::Updated { value } if value == &new_value,
            => DomainConfigEvent::Updated { .. }
        );

        self.events
            .push(DomainConfigEvent::Updated { value: new_value });

        Idempotent::Executed(())
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
                value: self.value,
            }],
        )
    }
}

#[cfg(test)]
mod tests {
    use es_entity::{IntoEvents as _, TryFromEvents as _};
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use crate::{DomainConfigId, DomainConfigKey, DomainConfigValue};

    use super::{DomainConfig, DomainConfigEvent, NewDomainConfig};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct SampleConfig {
        enabled: bool,
        limit: u32,
    }

    impl DomainConfigValue for SampleConfig {
        const KEY: DomainConfigKey = DomainConfigKey::new("sample-config");
    }

    fn build_config(id: DomainConfigId, value: &SampleConfig) -> DomainConfig {
        let events = NewDomainConfig::builder()
            .id(id)
            .key(SampleConfig::KEY)
            .value(serde_json::to_value(value).unwrap())
            .build()
            .unwrap()
            .into_events();

        DomainConfig::try_from_events(events).unwrap()
    }

    #[test]
    fn rehydrates_from_initialized_event() {
        let id = DomainConfigId::new();
        let value = SampleConfig {
            enabled: true,
            limit: 10,
        };

        let config = build_config(id, &value);

        assert_eq!(config.id, id);
        assert_eq!(config.key, SampleConfig::KEY);
        assert_eq!(config.current_value::<SampleConfig>().unwrap(), value);
    }

    #[test]
    fn apply_update_appends_event_and_updates_value() {
        let mut config = build_config(
            DomainConfigId::new(),
            &SampleConfig {
                enabled: true,
                limit: 5,
            },
        );
        let updated = SampleConfig {
            enabled: false,
            limit: 15,
        };

        let updated_json = json!(updated.clone());
        let result = config.update(updated_json.clone());

        assert!(result.did_execute());
        assert_eq!(config.events.iter_all().count(), 2);
        let last_event = config.events.iter_all().next_back().unwrap();
        assert!(matches!(
            last_event,
            DomainConfigEvent::Updated { value } if value == &updated_json
        ));
        assert_eq!(config.current_value::<SampleConfig>().unwrap(), updated);
    }

    #[test]
    fn apply_update_is_idempotent_when_value_is_unchanged() {
        let mut config = build_config(
            DomainConfigId::new(),
            &SampleConfig {
                enabled: true,
                limit: 5,
            },
        );
        let updated = SampleConfig {
            enabled: false,
            limit: 15,
        };

        let updated_json = json!(updated.clone());

        assert!(config.update(updated_json.clone()).did_execute());
        let result = config.update(updated_json.clone());

        assert!(result.was_ignored());
        assert_eq!(config.events.iter_all().count(), 2);
        assert_eq!(config.current_value::<SampleConfig>().unwrap(), updated);
    }

    #[test]
    fn rehydrates_after_multiple_updates() {
        let mut config = build_config(
            DomainConfigId::new(),
            &SampleConfig {
                enabled: true,
                limit: 5,
            },
        );

        let first = SampleConfig {
            enabled: false,
            limit: 6,
        };
        let second = SampleConfig {
            enabled: true,
            limit: 7,
        };

        let first_json = json!(first);
        let second_json = json!(second.clone());

        assert!(config.update(first_json.clone()).did_execute());
        assert!(config.update(second_json.clone()).did_execute());

        let rehydrated = DomainConfig::try_from_events(config.events.clone()).unwrap();

        assert_eq!(rehydrated.current_value::<SampleConfig>().unwrap(), second);
        assert_eq!(rehydrated.events.iter_all().count(), 3);
    }
}
