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

    use crate::{
        Complex, ConfigSpec, DomainConfigError, DomainConfigId, DomainConfigKey, ValueKind,
        Visibility,
    };

    use super::{DomainConfig, DomainConfigEvent, NewDomainConfig};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    struct SampleComplexConfig {
        enabled: bool,
        limit: u32,
    }

    struct SampleComplexConfigSpec;
    impl ConfigSpec for SampleComplexConfigSpec {
        const KEY: DomainConfigKey = DomainConfigKey::new("sample-config");
        const VISIBILITY: Visibility = Visibility::Exposed;
        type Kind = Complex<SampleComplexConfig>;

        fn validate(value: &SampleComplexConfig) -> Result<(), DomainConfigError> {
            if value.limit > 100 {
                return Err(DomainConfigError::InvalidState(
                    "Limit is too high".to_string(),
                ));
            }

            Ok(())
        }
    }

    struct SampleSimpleBoolSpec;
    impl ConfigSpec for SampleSimpleBoolSpec {
        const KEY: DomainConfigKey = DomainConfigKey::new("simple-bool");
        const VISIBILITY: Visibility = Visibility::Internal;
        type Kind = crate::Simple<bool>;
    }

    struct SampleExposedBoolSpec;
    impl ConfigSpec for SampleExposedBoolSpec {
        const KEY: DomainConfigKey = DomainConfigKey::new("simple-bool");
        const VISIBILITY: Visibility = Visibility::Exposed;
        type Kind = crate::Simple<bool>;
    }

    #[test]
    fn new_domain_config_emits_expected_init_metadata_for_sample_config() {
        let id = DomainConfigId::new();
        let value = SampleComplexConfig {
            enabled: true,
            limit: 10,
        };

        let events = NewDomainConfig::builder()
            .with_value::<SampleComplexConfigSpec>(id, value)
            .unwrap()
            .build()
            .unwrap()
            .into_events();

        let init = events.iter_all().next().expect("init event exists");
        let expected_type = <<SampleComplexConfigSpec as ConfigSpec>::Kind as ValueKind>::TYPE;
        assert!(matches!(
            init,
            DomainConfigEvent::Initialized {
                id: event_id,
                key,
                config_type,
                visibility,
                ..
            } if event_id == &id
                && key == &SampleComplexConfigSpec::KEY
                && config_type == &expected_type
                && visibility == &SampleComplexConfigSpec::VISIBILITY
        ));
    }

    #[test]
    fn new_domain_config_emits_expected_init_metadata_for_simple_bool() {
        let id = DomainConfigId::new();
        let events = NewDomainConfig::builder()
            .with_value::<SampleSimpleBoolSpec>(id, true)
            .unwrap()
            .build()
            .unwrap()
            .into_events();

        let init = events.iter_all().next().expect("init event exists");
        let expected_type = <<SampleSimpleBoolSpec as ConfigSpec>::Kind as ValueKind>::TYPE;
        assert!(matches!(
            init,
            DomainConfigEvent::Initialized {
                id: event_id,
                key,
                config_type,
                visibility,
                ..
            } if event_id == &id
                && key == &SampleSimpleBoolSpec::KEY
                && config_type == &expected_type
                && visibility == &SampleSimpleBoolSpec::VISIBILITY
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
            .with_value::<SampleComplexConfigSpec>(id, value.clone())
            .unwrap()
            .build()
            .unwrap()
            .into_events();
        let config = DomainConfig::try_from_events(events).unwrap();

        assert_eq!(config.id, id);
        assert_eq!(config.key, SampleComplexConfigSpec::KEY);
        assert_eq!(
            config.config_type,
            <<SampleComplexConfigSpec as ConfigSpec>::Kind as ValueKind>::TYPE
        );
        assert_eq!(config.visibility, SampleComplexConfigSpec::VISIBILITY);
        assert_eq!(
            config.current_value::<SampleComplexConfigSpec>().unwrap(),
            value
        );
    }

    #[test]
    fn rehydrates_simple_bool_from_events() {
        let id = DomainConfigId::new();
        let events = NewDomainConfig::builder()
            .with_value::<SampleSimpleBoolSpec>(id, false)
            .unwrap()
            .build()
            .unwrap()
            .into_events();
        let config = DomainConfig::try_from_events(events).unwrap();

        assert_eq!(config.id, id);
        assert_eq!(config.key, SampleSimpleBoolSpec::KEY);
        assert_eq!(
            config.config_type,
            <<SampleSimpleBoolSpec as ConfigSpec>::Kind as ValueKind>::TYPE
        );
        assert_eq!(config.visibility, SampleSimpleBoolSpec::VISIBILITY);
        assert!(!config.current_value::<SampleSimpleBoolSpec>().unwrap());
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
            .with_value::<SampleComplexConfigSpec>(id, initial)
            .unwrap()
            .build()
            .unwrap()
            .into_events();
        let mut config = DomainConfig::try_from_events(events).unwrap();

        assert!(
            config
                .update_value::<SampleComplexConfigSpec>(updated.clone())
                .expect("first update should succeed")
                .did_execute()
        );

        let result = config
            .update_value::<SampleComplexConfigSpec>(updated.clone())
            .expect("second update should not error");
        assert!(result.was_already_applied());

        let updated_json =
            <<SampleComplexConfigSpec as ConfigSpec>::Kind as ValueKind>::encode(&updated)
                .expect("value encodes");
        let last_event = config.events.iter_all().next_back().unwrap();
        assert!(matches!(
            last_event,
            DomainConfigEvent::Updated { value } if value == &updated_json
        ));
        assert_eq!(
            config.current_value::<SampleComplexConfigSpec>().unwrap(),
            updated
        );
    }

    #[test]
    fn update_value_is_idempotent_for_simple_bool() {
        let id = DomainConfigId::new();
        let events = NewDomainConfig::builder()
            .with_value::<SampleSimpleBoolSpec>(id, false)
            .unwrap()
            .build()
            .unwrap()
            .into_events();
        let mut config = DomainConfig::try_from_events(events).unwrap();

        assert!(
            config
                .update_value::<SampleSimpleBoolSpec>(true)
                .expect("first update should succeed")
                .did_execute()
        );

        let result = config
            .update_value::<SampleSimpleBoolSpec>(true)
            .expect("second update should not error");
        assert!(result.was_already_applied());

        let updated_json = <<SampleSimpleBoolSpec as ConfigSpec>::Kind as ValueKind>::encode(&true)
            .expect("value encodes");
        let last_event = config.events.iter_all().next_back().unwrap();
        assert!(matches!(
            last_event,
            DomainConfigEvent::Updated { value } if value == &updated_json
        ));
        assert!(config.current_value::<SampleSimpleBoolSpec>().unwrap());
    }

    #[test]
    fn create_rejects_invalid_sample_config() {
        let invalid = SampleComplexConfig {
            enabled: true,
            limit: 101,
        };

        let create_result = NewDomainConfig::builder()
            .with_value::<SampleComplexConfigSpec>(DomainConfigId::new(), invalid);
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
            .with_value::<SampleComplexConfigSpec>(
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

        let update_result = config.update_value::<SampleComplexConfigSpec>(invalid);
        assert!(
            matches!(update_result, Err(DomainConfigError::InvalidState(_))),
            "invalid update should fail validation"
        );
    }

    #[test]
    fn current_value_rejects_wrong_type() {
        let events = NewDomainConfig::builder()
            .with_value::<SampleSimpleBoolSpec>(DomainConfigId::new(), true)
            .unwrap()
            .build()
            .unwrap()
            .into_events();
        let config = DomainConfig::try_from_events(events).unwrap();

        let read_type_error = config.current_value::<SampleComplexConfigSpec>();
        assert!(matches!(
            read_type_error,
            Err(DomainConfigError::InvalidType(message)) if message.contains("config type")
        ));
    }

    #[test]
    fn current_value_rejects_wrong_visibility() {
        let events = NewDomainConfig::builder()
            .with_value::<SampleSimpleBoolSpec>(DomainConfigId::new(), true)
            .unwrap()
            .build()
            .unwrap()
            .into_events();
        let config = DomainConfig::try_from_events(events).unwrap();

        let read_visibility_error = config.current_value::<SampleExposedBoolSpec>();
        assert!(matches!(
            read_visibility_error,
            Err(DomainConfigError::InvalidType(message)) if message.contains("visibility")
        ));
    }

    #[test]
    fn update_rejects_wrong_type() {
        let events = NewDomainConfig::builder()
            .with_value::<SampleSimpleBoolSpec>(DomainConfigId::new(), true)
            .unwrap()
            .build()
            .unwrap()
            .into_events();
        let mut config = DomainConfig::try_from_events(events).unwrap();

        let update_type_error =
            config.update_value::<SampleComplexConfigSpec>(SampleComplexConfig {
                enabled: true,
                limit: 1,
            });
        assert!(matches!(
            update_type_error,
            Err(DomainConfigError::InvalidType(message)) if message.contains("config type")
        ));
    }
}
