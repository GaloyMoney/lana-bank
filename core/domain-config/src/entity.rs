use derive_builder::Builder;
use es_entity::*;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ConfigSpec, ConfigType, DomainConfigError, ValueKind,
    primitives::{DomainConfigId, DomainConfigKey, Visibility},
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
        visibility: Visibility,
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
    pub visibility: Visibility,
    events: EntityEvents<DomainConfigEvent>,
}

impl DomainConfig {
    pub(super) fn current_value<C>(
        &self,
    ) -> Result<<C::Kind as ValueKind>::Value, DomainConfigError>
    where
        C: ConfigSpec,
    {
        self.ensure::<C>()?;
        <C::Kind as ValueKind>::decode(self.current_json_value().clone())
    }

    pub(super) fn update_value<C>(
        &mut self,
        new_value: <C::Kind as ValueKind>::Value,
    ) -> Result<Idempotent<()>, DomainConfigError>
    where
        C: ConfigSpec,
    {
        self.ensure::<C>()?;
        C::validate(&new_value)?;

        let value_json = <C::Kind as ValueKind>::encode(&new_value)?;
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

    fn ensure<C: ConfigSpec>(&self) -> Result<(), DomainConfigError> {
        let expected_type = <C::Kind as ValueKind>::TYPE;
        if self.config_type != expected_type {
            return Err(DomainConfigError::InvalidType(format!(
                "Invalid config type for {key}: expected {expected}, found {found}",
                key = self.key,
                expected = expected_type,
                found = self.config_type
            )));
        }

        if self.visibility != C::VISIBILITY {
            return Err(DomainConfigError::InvalidType(format!(
                "Invalid visibility for {key}: expected {expected}, found {found}",
                key = self.key,
                expected = C::VISIBILITY,
                found = self.visibility
            )));
        }

        Ok(())
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
                    visibility,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .key(key.clone())
                        .config_type(*config_type)
                        .visibility(*visibility);
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
    pub(super) visibility: Visibility,
    value: serde_json::Value,
}

impl NewDomainConfig {
    pub fn builder() -> NewDomainConfigBuilder {
        NewDomainConfigBuilder::default()
    }
}

impl NewDomainConfigBuilder {
    pub fn with_value<C>(
        mut self,
        id: DomainConfigId,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<Self, DomainConfigError>
    where
        C: ConfigSpec,
    {
        C::validate(&value)?;
        let value_json = <C::Kind as ValueKind>::encode(&value)?;
        let config_type = <C::Kind as ValueKind>::TYPE;

        self.id(id);
        self.key(C::KEY);
        self.config_type(config_type);
        self.visibility(C::VISIBILITY);
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
                config_type: self.config_type,
                visibility: self.visibility,
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
        Complex, ConfigSpec, ConfigType, DomainConfigError, DomainConfigId, DomainConfigKey,
        Visibility,
    };

    use super::{DomainConfig, DomainConfigEvent, NewDomainConfig};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    struct SampleConfig {
        enabled: bool,
        limit: u32,
    }

    struct SampleConfigSpec;
    impl ConfigSpec for SampleConfigSpec {
        const KEY: DomainConfigKey = DomainConfigKey::new("sample-config");
        const VISIBILITY: Visibility = Visibility::Exposed;
        type Kind = Complex<SampleConfig>;

        fn validate(value: &SampleConfig) -> Result<(), DomainConfigError> {
            if value.limit > 100 {
                return Err(DomainConfigError::InvalidState(
                    "Limit is too high".to_string(),
                ));
            }

            Ok(())
        }
    }

    struct SimpleBoolSpec;
    impl ConfigSpec for SimpleBoolSpec {
        const KEY: DomainConfigKey = DomainConfigKey::new("simple-bool");
        const VISIBILITY: Visibility = Visibility::Internal;
        type Kind = crate::Simple<bool>;
    }

    fn build_exposed_config(id: DomainConfigId, value: &SampleConfig) -> DomainConfig {
        let events = NewDomainConfig::builder()
            .with_value::<SampleConfigSpec>(id, value.clone())
            .unwrap()
            .build()
            .unwrap()
            .into_events();

        DomainConfig::try_from_events(events).unwrap()
    }

    #[test]
    fn exposed_init_sets_fields() {
        let id = DomainConfigId::new();
        let value = SampleConfig {
            enabled: true,
            limit: 10,
        };

        let events = NewDomainConfig::builder()
            .with_value::<SampleConfigSpec>(id, value)
            .unwrap()
            .build()
            .unwrap()
            .into_events();

        let init = events.iter_all().next().unwrap();
        assert!(matches!(
            init,
            DomainConfigEvent::Initialized {
                config_type,
                visibility,
                ..
            } if *config_type == ConfigType::Complex && *visibility == Visibility::Exposed
        ));
    }

    #[test]
    fn internal_init_sets_fields() {
        let id = DomainConfigId::new();
        let events = NewDomainConfig::builder()
            .with_value::<SimpleBoolSpec>(id, true)
            .unwrap()
            .build()
            .unwrap()
            .into_events();

        let init = events.iter_all().next().unwrap();
        assert!(matches!(
            init,
            DomainConfigEvent::Initialized {
                config_type,
                visibility,
                ..
            } if *config_type == ConfigType::Bool && *visibility == Visibility::Internal
        ));
    }

    #[test]
    fn rehydrates_exposed_config() {
        let id = DomainConfigId::new();
        let value = SampleConfig {
            enabled: true,
            limit: 10,
        };

        let config = build_exposed_config(id, &value);

        assert_eq!(config.id, id);
        assert_eq!(config.key, <SampleConfigSpec as ConfigSpec>::KEY);
        assert_eq!(config.config_type, ConfigType::Complex);
        assert_eq!(config.visibility, Visibility::Exposed);
        assert_eq!(config.current_value::<SampleConfigSpec>().unwrap(), value);
    }

    #[test]
    fn rehydrates_internal_config() {
        let id = DomainConfigId::new();
        let value = false;
        let events = NewDomainConfig::builder()
            .with_value::<SimpleBoolSpec>(id, value)
            .unwrap()
            .build()
            .unwrap()
            .into_events();

        let config = DomainConfig::try_from_events(events).unwrap();

        assert_eq!(config.config_type, ConfigType::Bool);
        assert_eq!(config.visibility, Visibility::Internal);
        assert_eq!(config.key, <SimpleBoolSpec as ConfigSpec>::KEY);
        let current_value = config.current_value::<SimpleBoolSpec>().unwrap();
        assert_eq!(current_value, value);
    }

    #[test]
    fn update_exposed_is_idempotent() {
        let mut config = build_exposed_config(
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
                .update_value::<SampleConfigSpec>(updated.clone())
                .expect("first update should succeed")
                .did_execute()
        );
        let result = config
            .update_value::<SampleConfigSpec>(updated.clone())
            .expect("second update should not error");

        assert!(result.was_already_applied());
        let last_event = config.events.iter_all().next_back().unwrap();
        assert!(matches!(
            last_event,
            DomainConfigEvent::Updated { value } if value == &updated_json
        ));
        assert_eq!(config.current_value::<SampleConfigSpec>().unwrap(), updated);
    }

    #[test]
    fn update_internal_is_idempotent() {
        let id = DomainConfigId::new();
        let initial_value = false;
        let new_value = true;

        let events = NewDomainConfig::builder()
            .with_value::<SimpleBoolSpec>(id, initial_value)
            .unwrap()
            .build()
            .unwrap()
            .into_events();

        let mut config = DomainConfig::try_from_events(events).unwrap();

        assert!(
            config
                .update_value::<SimpleBoolSpec>(new_value)
                .unwrap()
                .did_execute()
        );
        assert!(
            config
                .update_value::<SimpleBoolSpec>(new_value)
                .unwrap()
                .was_already_applied()
        );
        let last_event = config.events.iter_all().next_back().unwrap();
        assert!(matches!(
            last_event,
            DomainConfigEvent::Updated { value } if value == &json!(new_value)
        ));
    }

    #[test]
    fn type_invariant_enforced() {
        let id = DomainConfigId::new();
        let events = NewDomainConfig::builder()
            .with_value::<SimpleBoolSpec>(id, true)
            .unwrap()
            .build()
            .unwrap()
            .into_events();
        let mut config = DomainConfig::try_from_events(events).unwrap();

        let result = config.current_value::<SampleConfigSpec>();
        assert!(matches!(result, Err(DomainConfigError::InvalidType(_))));

        let result = config.update_value::<SampleConfigSpec>(SampleConfig {
            enabled: true,
            limit: 1,
        });
        assert!(matches!(result, Err(DomainConfigError::InvalidType(_))));

        let config = build_exposed_config(
            DomainConfigId::new(),
            &SampleConfig {
                enabled: true,
                limit: 1,
            },
        );
        let result = config.current_value::<SimpleBoolSpec>();
        assert!(matches!(result, Err(DomainConfigError::InvalidType(_))));
    }
}
