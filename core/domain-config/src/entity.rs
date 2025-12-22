use derive_builder::Builder;
use es_entity::*;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ComplexConfig, ConfigType, DomainConfigError, SimpleConfig, SimpleEntry, SimpleScalar,
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
        config_type: ConfigType,
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
    pub config_type: ConfigType,
    events: EntityEvents<DomainConfigEvent>,
}

impl DomainConfig {
    pub(super) fn update_complex<T>(
        &mut self,
        new_value: T,
    ) -> Result<Idempotent<()>, DomainConfigError>
    where
        T: ComplexConfig,
    {
        self.ensure_complex()?;
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

    pub(super) fn current_complex_value<T>(&self) -> Result<T, DomainConfigError>
    where
        T: ComplexConfig,
    {
        self.ensure_complex()?;
        let value = self.current_json_value();
        Ok(serde_json::from_value(value.clone())?)
    }

    pub(super) fn current_simple_value<T>(&self) -> Result<T::Scalar, DomainConfigError>
    where
        T: SimpleConfig,
    {
        self.ensure_config_type(T::Scalar::CONFIG_TYPE)?;
        T::Scalar::from_json(self.current_json_value().clone())
    }

    pub(super) fn into_simple_entry(self) -> Result<SimpleEntry, DomainConfigError> {
        let value_json = self.current_json_value().clone();
        if !self.config_type.is_simple() {
            return Err(DomainConfigError::InvalidType(format!(
                "Config is not simple for {key}",
                key = self.key
            )));
        }
        let value = self.config_type.format_json_value(&value_json)?;

        Ok(SimpleEntry {
            key: self.key.clone(),
            config_type: self.config_type,
            value,
        })
    }

    pub(super) fn update_simple<T>(
        &mut self,
        new_value: T,
    ) -> Result<Idempotent<()>, DomainConfigError>
    where
        T: crate::SimpleScalar,
    {
        let value_json = T::to_json(&new_value);

        self.ensure_config_type(T::CONFIG_TYPE)?;
        idempotency_guard!(
            self.events.iter_all().rev(),
            DomainConfigEvent::Updated { value } if value == &value_json,
            => DomainConfigEvent::Updated { .. }
        );

        self.events
            .push(DomainConfigEvent::Updated { value: value_json });

        Ok(Idempotent::Executed(()))
    }

    fn current_json_value(&self) -> &serde_json::Value {
        let last_event = self
            .events
            .iter_all()
            .next_back()
            .expect("last event exists");
        match last_event {
            DomainConfigEvent::Initialized { value, .. } => value,
            DomainConfigEvent::Updated { value } => value,
        }
    }

    fn ensure_config_type(&self, expected: ConfigType) -> Result<(), DomainConfigError> {
        match self.config_type {
            found if found == expected => Ok(()),
            found => Err(DomainConfigError::InvalidType(format!(
                "Invalid config type for {key}: expected {expected}, found {found}",
                key = self.key
            ))),
        }
    }

    fn ensure_complex(&self) -> Result<(), DomainConfigError> {
        match self.config_type {
            ConfigType::Complex => Ok(()),
            found => Err(DomainConfigError::InvalidType(format!(
                "Config is simple for {key}: found config type {found}",
                key = self.key
            ))),
        }
    }
}

impl TryFromEvents<DomainConfigEvent> for DomainConfig {
    fn try_from_events(events: EntityEvents<DomainConfigEvent>) -> Result<Self, EsEntityError> {
        let mut builder = DomainConfigBuilder::default();

        for event in events.iter_all() {
            match event {
                DomainConfigEvent::Initialized {
                    id,
                    key,
                    config_type,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .key(key.clone())
                        .config_type(*config_type);
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
    pub(super) config_type: ConfigType,
    value: serde_json::Value,
}

impl NewDomainConfig {
    pub fn builder() -> NewDomainConfigBuilder {
        NewDomainConfigBuilder::default()
    }
}

impl NewDomainConfigBuilder {
    pub fn with_complex_value<T>(
        mut self,
        id: DomainConfigId,
        value: T,
    ) -> Result<Self, DomainConfigError>
    where
        T: ComplexConfig,
    {
        value.validate()?;
        let value_json = serde_json::to_value(value)?;

        self.id(id);
        self.key(T::KEY);
        self.config_type(ConfigType::Complex);
        self.value(value_json);

        Ok(self)
    }

    pub fn with_simple<T>(
        mut self,
        id: DomainConfigId,
        value: T::Scalar,
    ) -> Result<Self, DomainConfigError>
    where
        T: SimpleConfig,
    {
        self.id(id);
        self.key(T::KEY);
        self.config_type(T::Scalar::CONFIG_TYPE);
        self.value(T::Scalar::to_json(&value));
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
                config_type: self.config_type,
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

    use crate::{
        ComplexConfig, ConfigType, DomainConfigError, DomainConfigId, DomainConfigKey, SimpleConfig,
    };

    use super::{DomainConfig, DomainConfigEvent, NewDomainConfig};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    struct SampleConfig {
        enabled: bool,
        limit: u32,
    }

    impl ComplexConfig for SampleConfig {
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

    struct SimpleBool;
    impl SimpleConfig for SimpleBool {
        type Scalar = bool;
        const KEY: DomainConfigKey = DomainConfigKey::new("simple-bool");
    }

    fn build_config(id: DomainConfigId, value: &SampleConfig) -> DomainConfig {
        let events = NewDomainConfig::builder()
            .with_complex_value(id, value.clone())
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
        assert_eq!(
            config.current_complex_value::<SampleConfig>().unwrap(),
            value
        );
    }

    #[test]
    fn new_domain_config_validates_value() {
        let invalid = SampleConfig {
            enabled: true,
            limit: 101,
        };

        let result = NewDomainConfig::builder().with_complex_value(DomainConfigId::new(), invalid);

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

        let result = config.update_complex(invalid.clone());

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
            .update_complex(updated.clone())
            .expect("update should succeed");

        assert!(result.did_execute());
        assert_eq!(config.events.iter_all().count(), 2);
        let last_event = config.events.iter_all().next_back().unwrap();
        assert!(matches!(
            last_event,
            DomainConfigEvent::Updated { value } if value == &updated_json
        ));
        assert_eq!(
            config.current_complex_value::<SampleConfig>().unwrap(),
            updated
        );
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
                .update_complex(updated.clone())
                .expect("first update should succeed")
                .did_execute()
        );
        let result = config
            .update_complex(updated.clone())
            .expect("second update should not error");

        assert!(result.was_already_applied());
        assert_eq!(config.events.iter_all().count(), 2);
        let last_event = config.events.iter_all().next_back().unwrap();
        assert!(matches!(
            last_event,
            DomainConfigEvent::Updated { value } if value == &updated_json
        ));
        assert_eq!(
            config.current_complex_value::<SampleConfig>().unwrap(),
            updated
        );
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
                .update_complex(first.clone())
                .expect("first update should succeed")
                .did_execute()
        );
        assert!(
            config
                .update_complex(second.clone())
                .expect("second update should succeed")
                .did_execute()
        );

        let rehydrated = DomainConfig::try_from_events(config.events.clone()).unwrap();

        assert_eq!(
            rehydrated.current_complex_value::<SampleConfig>().unwrap(),
            second
        );
        assert_eq!(rehydrated.events.iter_all().count(), 3);
    }

    #[test]
    fn builder_sets_config_type() {
        let id = DomainConfigId::new();
        let expected_key = DomainConfigKey::new("simple-bool");

        let new = NewDomainConfig::builder()
            .with_simple::<SimpleBool>(id, true)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(new.config_type, ConfigType::Bool);
        assert_eq!(new.key, expected_key);
    }

    #[test]
    fn rehydrates_config_type_from_event() {
        let id = DomainConfigId::new();
        let value = true;
        let expected_key = DomainConfigKey::new("simple-bool");

        let events = NewDomainConfig::builder()
            .with_simple::<SimpleBool>(id, value)
            .unwrap()
            .build()
            .unwrap()
            .into_events();

        let config = DomainConfig::try_from_events(events).unwrap();

        assert_eq!(config.config_type, ConfigType::Bool);
        let entry = config
            .into_simple_entry()
            .expect("should parse simple entry");
        assert_eq!(entry.key, expected_key);
        assert_eq!(entry.value, "true");
    }

    #[test]
    fn update_simple_is_idempotent() {
        let id = DomainConfigId::new();
        let initial_value = false;
        let new_value = true;

        let events = NewDomainConfig::builder()
            .with_simple::<SimpleBool>(id, initial_value)
            .unwrap()
            .build()
            .unwrap()
            .into_events();

        let mut config = DomainConfig::try_from_events(events).unwrap();

        assert!(config.update_simple(new_value).unwrap().did_execute());
        assert!(config.update_simple(new_value).unwrap().was_ignored());
        let last_event = config.events.iter_all().next_back().unwrap();
        assert!(matches!(
            last_event,
            DomainConfigEvent::Updated { value } if value == &json!(new_value)
        ));
    }
}
