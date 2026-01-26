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
    pub(super) fn current_value<C>(&self) -> Option<<C::Kind as ValueKind>::Value>
    where
        C: ConfigSpec,
    {
        self.ensure::<C>().ok()?;
        let value = self.current_json_value();
        if value.is_null() {
            return None;
        }
        <C::Kind as ValueKind>::decode(value.clone()).ok()
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
        self.update_json_value(value_json)
    }

    pub(super) fn apply_exposed_update_from_json(
        &mut self,
        entry: &crate::registry::ConfigSpecEntry,
        new_value: serde_json::Value,
    ) -> Result<Idempotent<()>, DomainConfigError> {
        if self.visibility != crate::Visibility::Exposed {
            return Err(DomainConfigError::InvalidState(format!(
                "Config {} is not exposed",
                self.key
            )));
        }

        if self.visibility != entry.visibility {
            return Err(DomainConfigError::InvalidState(format!(
                "Invalid visibility for {}: expected {}, found={}",
                self.key, entry.visibility, self.visibility
            )));
        }

        if self.config_type != entry.config_type {
            return Err(DomainConfigError::InvalidType(format!(
                "Invalid config type for {}: expected {}, found {}",
                self.key, entry.config_type, self.config_type
            )));
        }

        (entry.validate_json)(&new_value)?;

        self.update_json_value(new_value)
    }

    /// Apply update from JSON for any config (CLI startup, no auth required).
    ///
    /// Unlike `apply_exposed_update_from_json`, this method works for any config
    /// regardless of visibility, for use during CLI startup before GraphQL is available.
    pub fn apply_update_from_json(
        &mut self,
        entry: &crate::registry::ConfigSpecEntry,
        new_value: serde_json::Value,
    ) -> Result<Idempotent<()>, DomainConfigError> {
        if self.config_type != entry.config_type {
            return Err(DomainConfigError::InvalidType(format!(
                "Invalid config type for {}: expected {}, found {}",
                self.key, entry.config_type, self.config_type
            )));
        }

        if self.visibility != entry.visibility {
            return Err(DomainConfigError::InvalidState(format!(
                "Invalid visibility for {}: expected {}, found {}",
                self.key, entry.visibility, self.visibility
            )));
        }

        (entry.validate_json)(&new_value)?;

        self.update_json_value(new_value)
    }

    fn update_json_value(
        &mut self,
        new_value: serde_json::Value,
    ) -> Result<Idempotent<()>, DomainConfigError> {
        if self.current_json_value() == &new_value {
            return Ok(Idempotent::AlreadyApplied);
        }

        self.events
            .push(DomainConfigEvent::Updated { value: new_value });

        Ok(Idempotent::Executed(()))
    }

    pub fn current_json_value(&self) -> &serde_json::Value {
        const NULL_JSON_VALUE: serde_json::Value = serde_json::Value::Null;
        let value = self.events.iter_all().rev().find_map(|event| match event {
            DomainConfigEvent::Updated { value } => Some(value),
            _ => None,
        });

        value.unwrap_or(&NULL_JSON_VALUE)
    }

    pub(super) fn ensure<C: ConfigSpec>(&self) -> Result<(), DomainConfigError> {
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
    #[builder(default)]
    value: Option<serde_json::Value>,
}

impl NewDomainConfig {
    pub fn builder() -> NewDomainConfigBuilder {
        NewDomainConfigBuilder::default()
    }
}

impl NewDomainConfigBuilder {
    pub fn seed(
        mut self,
        id: DomainConfigId,
        key: DomainConfigKey,
        config_type: ConfigType,
        visibility: Visibility,
    ) -> Self {
        self.id(id);
        self.key(key);
        self.config_type(config_type);
        self.visibility(visibility);

        self
    }

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
        self.value(Some(value_json));

        Ok(self)
    }
}

impl IntoEvents<DomainConfigEvent> for NewDomainConfig {
    fn into_events(self) -> EntityEvents<DomainConfigEvent> {
        let mut events = Vec::new();
        events.push(DomainConfigEvent::Initialized {
            id: self.id,
            key: self.key,
            config_type: self.config_type,
            visibility: self.visibility,
        });

        if let Some(value) = self.value {
            events.push(DomainConfigEvent::Updated { value });
        }

        EntityEvents::init(self.id, events)
    }
}

#[cfg(test)]
mod tests {
    use es_entity::{IntoEvents as _, TryFromEvents as _};
    use serde::{Deserialize, Serialize};

    use crate::{ConfigSpec, DomainConfigError, DomainConfigId, ValueKind};

    use super::{DomainConfig, DomainConfigEvent, NewDomainConfig};

    crate::define_internal_config! {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
        struct SampleComplexConfig {
            enabled: bool,
            limit: u32,
        }

        spec {
            key: "sample-config";
            validate: |value: &Self| {
                if value.limit > 100 {
                    return Err(DomainConfigError::InvalidState(
                        "Limit is too high".to_string(),
                    ));
                }

                Ok(())
            };
        }
    }

    crate::define_internal_config! {
        #[allow(dead_code)]
        struct SampleSimpleBool(bool);
        spec {
            key: "simple-bool";
        }
    }

    crate::define_exposed_config! {
        #[allow(dead_code)]
        struct SampleExposedBool(bool);
        spec {
            key: "simple-bool";
        }
    }

    #[test]
    fn new_domain_config_emits_expected_init_metadata_for_sample_config() {
        let id = DomainConfigId::new();
        let value = SampleComplexConfig {
            enabled: true,
            limit: 10,
        };

        let events = NewDomainConfig::builder()
            .with_value::<SampleComplexConfig>(id, value)
            .unwrap()
            .build()
            .unwrap()
            .into_events();

        let init = events.iter_all().next().expect("init event exists");
        let expected_type = <<SampleComplexConfig as ConfigSpec>::Kind as ValueKind>::TYPE;
        assert!(matches!(
            init,
            DomainConfigEvent::Initialized {
                id: event_id,
                key,
                config_type,
                visibility,
                ..
            } if event_id == &id
                && key == &SampleComplexConfig::KEY
                && config_type == &expected_type
                && visibility == &SampleComplexConfig::VISIBILITY
        ));
    }

    #[test]
    fn new_domain_config_emits_expected_init_metadata_for_simple_bool() {
        let id = DomainConfigId::new();
        let events = NewDomainConfig::builder()
            .with_value::<SampleSimpleBool>(id, true)
            .unwrap()
            .build()
            .unwrap()
            .into_events();

        let init = events.iter_all().next().expect("init event exists");
        let expected_type = <<SampleSimpleBool as ConfigSpec>::Kind as ValueKind>::TYPE;
        assert!(matches!(
            init,
            DomainConfigEvent::Initialized {
                id: event_id,
                key,
                config_type,
                visibility,
                ..
            } if event_id == &id
                && key == &SampleSimpleBool::KEY
                && config_type == &expected_type
                && visibility == &SampleSimpleBool::VISIBILITY
        ));
    }

    #[test]
    fn rehydrates_sample_config_from_events() {
        let id = DomainConfigId::new();
        let value = SampleComplexConfig {
            enabled: true,
            limit: 10,
        };

        let events = NewDomainConfig::builder()
            .with_value::<SampleComplexConfig>(id, value.clone())
            .unwrap()
            .build()
            .unwrap()
            .into_events();
        let config = DomainConfig::try_from_events(events).unwrap();

        assert_eq!(config.id, id);
        assert_eq!(config.key, SampleComplexConfig::KEY);
        assert_eq!(
            config.config_type,
            <<SampleComplexConfig as ConfigSpec>::Kind as ValueKind>::TYPE
        );
        assert_eq!(config.visibility, SampleComplexConfig::VISIBILITY);
        assert_eq!(
            config.current_value::<SampleComplexConfig>().unwrap(),
            value
        );
    }

    #[test]
    fn rehydrates_simple_bool_from_events() {
        let id = DomainConfigId::new();
        let events = NewDomainConfig::builder()
            .with_value::<SampleSimpleBool>(id, false)
            .unwrap()
            .build()
            .unwrap()
            .into_events();
        let config = DomainConfig::try_from_events(events).unwrap();

        assert_eq!(config.id, id);
        assert_eq!(config.key, SampleSimpleBool::KEY);
        assert_eq!(
            config.config_type,
            <<SampleSimpleBool as ConfigSpec>::Kind as ValueKind>::TYPE
        );
        assert_eq!(config.visibility, SampleSimpleBool::VISIBILITY);
        assert!(!config.current_value::<SampleSimpleBool>().unwrap());
    }

    #[test]
    fn update_value_is_idempotent_for_sample_config() {
        let id = DomainConfigId::new();
        let initial = SampleComplexConfig {
            enabled: true,
            limit: 5,
        };
        let updated = SampleComplexConfig {
            enabled: false,
            limit: 15,
        };

        let events = NewDomainConfig::builder()
            .with_value::<SampleComplexConfig>(id, initial)
            .unwrap()
            .build()
            .unwrap()
            .into_events();
        let mut config = DomainConfig::try_from_events(events).unwrap();

        assert!(
            config
                .update_value::<SampleComplexConfig>(updated.clone())
                .expect("first update should succeed")
                .did_execute()
        );

        let result = config
            .update_value::<SampleComplexConfig>(updated.clone())
            .expect("second update should not error");
        assert!(result.was_already_applied());

        let updated_json =
            <<SampleComplexConfig as ConfigSpec>::Kind as ValueKind>::encode(&updated)
                .expect("value encodes");
        let last_event = config.events.iter_all().next_back().unwrap();
        assert!(matches!(
            last_event,
            DomainConfigEvent::Updated { value } if value == &updated_json
        ));
        assert_eq!(
            config.current_value::<SampleComplexConfig>().unwrap(),
            updated
        );
    }

    #[test]
    fn update_value_is_idempotent_for_simple_bool() {
        let id = DomainConfigId::new();
        let events = NewDomainConfig::builder()
            .with_value::<SampleSimpleBool>(id, false)
            .unwrap()
            .build()
            .unwrap()
            .into_events();
        let mut config = DomainConfig::try_from_events(events).unwrap();

        assert!(
            config
                .update_value::<SampleSimpleBool>(true)
                .expect("first update should succeed")
                .did_execute()
        );

        let result = config
            .update_value::<SampleSimpleBool>(true)
            .expect("second update should not error");
        assert!(result.was_already_applied());

        let updated_json = <<SampleSimpleBool as ConfigSpec>::Kind as ValueKind>::encode(&true)
            .expect("value encodes");
        let last_event = config.events.iter_all().next_back().unwrap();
        assert!(matches!(
            last_event,
            DomainConfigEvent::Updated { value } if value == &updated_json
        ));
        assert!(config.current_value::<SampleSimpleBool>().unwrap());
    }

    #[test]
    fn create_rejects_invalid_sample_config() {
        let invalid = SampleComplexConfig {
            enabled: true,
            limit: 101,
        };

        let create_result = NewDomainConfig::builder()
            .with_value::<SampleComplexConfig>(DomainConfigId::new(), invalid);
        assert!(
            matches!(create_result, Err(DomainConfigError::InvalidState(_))),
            "invalid value should fail validation"
        );
    }

    #[test]
    fn update_rejects_invalid_sample_config() {
        let invalid = SampleComplexConfig {
            enabled: true,
            limit: 101,
        };

        let events = NewDomainConfig::builder()
            .with_value::<SampleComplexConfig>(
                DomainConfigId::new(),
                SampleComplexConfig {
                    enabled: true,
                    limit: 10,
                },
            )
            .unwrap()
            .build()
            .unwrap()
            .into_events();
        let mut config = DomainConfig::try_from_events(events).unwrap();

        let update_result = config.update_value::<SampleComplexConfig>(invalid);
        assert!(
            matches!(update_result, Err(DomainConfigError::InvalidState(_))),
            "invalid update should fail validation"
        );
    }

    #[test]
    fn current_value_rejects_wrong_type() {
        let events = NewDomainConfig::builder()
            .with_value::<SampleSimpleBool>(DomainConfigId::new(), true)
            .unwrap()
            .build()
            .unwrap()
            .into_events();
        let config = DomainConfig::try_from_events(events).unwrap();

        let read_type = config.current_value::<SampleComplexConfig>();
        assert!(read_type.is_none());
    }

    #[test]
    fn current_value_rejects_wrong_visibility() {
        let events = NewDomainConfig::builder()
            .with_value::<SampleSimpleBool>(DomainConfigId::new(), true)
            .unwrap()
            .build()
            .unwrap()
            .into_events();
        let config = DomainConfig::try_from_events(events).unwrap();

        let read_visibility = config.current_value::<SampleExposedBool>();
        assert!(read_visibility.is_none());
    }

    #[test]
    fn update_rejects_wrong_type() {
        let events = NewDomainConfig::builder()
            .with_value::<SampleSimpleBool>(DomainConfigId::new(), true)
            .unwrap()
            .build()
            .unwrap()
            .into_events();
        let mut config = DomainConfig::try_from_events(events).unwrap();

        let update_type_error = config.update_value::<SampleComplexConfig>(SampleComplexConfig {
            enabled: true,
            limit: 1,
        });
        assert!(matches!(
            update_type_error,
            Err(DomainConfigError::InvalidType(message)) if message.contains("config type")
        ));
    }
}
