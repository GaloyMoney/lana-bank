use derive_builder::Builder;
use es_entity::*;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    DomainConfigError, DomainConfigValue,
    primitives::{DomainConfigId, DomainConfigKey},
};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
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
    pub(super) fn update<T>(&mut self, new_value: T) -> Result<Idempotent<()>, DomainConfigError>
    where
        T: DomainConfigValue,
    {
        new_value.validate()?;

        let value_json = serde_json::to_value(new_value)?;
        idempotency_guard!(
            self.events.iter_all().rev(),
            DomainConfigEvent::Updated { value } if value == &value_json,
            => DomainConfigEvent::Updated { .. }
        );

        self.events
            .push(DomainConfigEvent::Updated { value: value_json });

        Ok(Idempotent::Executed(()))
    }

    pub(super) fn current_value<T>(&self) -> Result<T, DomainConfigError>
    where
        T: DomainConfigValue,
    {
        let last_event = self
            .events
            .iter_all()
            .next_back()
            .expect("last event exists");
        let value = match last_event {
            DomainConfigEvent::Initialized { value, .. } => value,
            DomainConfigEvent::Updated { value } => value,
        };
        Ok(serde_json::from_value(value.clone())?)
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

impl NewDomainConfigBuilder {
    pub fn with_value<T>(mut self, id: DomainConfigId, value: T) -> Result<Self, DomainConfigError>
    where
        T: DomainConfigValue,
    {
        value.validate()?;
        let value_json = serde_json::to_value(value)?;

        self.id(id);
        self.key(T::KEY);
        self.value(value_json);

        Ok(self)
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

    use crate::{DomainConfigError, DomainConfigId, DomainConfigKey, DomainConfigValue};

    use super::{DomainConfig, DomainConfigEvent, NewDomainConfig};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    struct SampleConfig {
        enabled: bool,
        limit: u32,
    }

    impl DomainConfigValue for SampleConfig {
        const KEY: DomainConfigKey = DomainConfigKey::new("sample-config");

        fn validate(&self) -> Result<(), DomainConfigError> {
            if self.limit > 100 {
                return Err(DomainConfigError::InvalidState(
                    "Limit is too high".to_string(),
                ));
            }

            Ok(())
        }
    }

    fn build_config(id: DomainConfigId, value: &SampleConfig) -> DomainConfig {
        let events = NewDomainConfig::builder()
            .with_value(id, value.clone())
            .unwrap()
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
    fn new_domain_config_validates_value() {
        let invalid = SampleConfig {
            enabled: true,
            limit: 101,
        };

        let result = NewDomainConfig::builder().with_value(DomainConfigId::new(), invalid);

        assert!(matches!(result, Err(DomainConfigError::InvalidState(_))));
    }

    #[test]
    fn update_domain_config_validates_value() {
        let mut config = build_config(
            DomainConfigId::new(),
            &SampleConfig {
                enabled: true,
                limit: 5,
            },
        );
        let invalid = SampleConfig {
            enabled: false,
            limit: 101,
        };

        let result = config.update(invalid.clone());

        assert!(matches!(result, Err(DomainConfigError::InvalidState(_))));
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
        let result = config
            .update(updated.clone())
            .expect("update should succeed");

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

        assert!(
            config
                .update(updated.clone())
                .expect("first update should succeed")
                .did_execute()
        );
        let result = config
            .update(updated.clone())
            .expect("second update should not error");

        assert!(result.was_already_applied());
        assert_eq!(config.events.iter_all().count(), 2);
        let last_event = config.events.iter_all().next_back().unwrap();
        assert!(matches!(
            last_event,
            DomainConfigEvent::Updated { value } if value == &updated_json
        ));
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

        assert!(
            config
                .update(first.clone())
                .expect("first update should succeed")
                .did_execute()
        );
        assert!(
            config
                .update(second.clone())
                .expect("second update should succeed")
                .did_execute()
        );

        let rehydrated = DomainConfig::try_from_events(config.events.clone()).unwrap();

        assert_eq!(rehydrated.current_value::<SampleConfig>().unwrap(), second);
        assert_eq!(rehydrated.events.iter_all().count(), 3);
    }
}
