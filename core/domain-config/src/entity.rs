use derive_builder::Builder;
use es_entity::*;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ConfigFlavor, ConfigSpec, ConfigType, DomainConfigError, DomainConfigFlavorEncrypted,
    DomainConfigFlavorPlaintext, ValueKind,
    encryption::EncryptionKey,
    primitives::{DomainConfigId, DomainConfigKey, Visibility},
    value::DomainConfigValue,
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
        encrypted: bool,
    },
    Updated {
        value: DomainConfigValue,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct DomainConfig {
    pub id: DomainConfigId,
    pub key: DomainConfigKey,
    pub config_type: ConfigType,
    pub visibility: Visibility,
    pub encrypted: bool,
    events: EntityEvents<DomainConfigEvent>,
}

impl DomainConfig {
    pub(super) fn current_value_plain<C>(&self) -> Option<<C::Kind as ValueKind>::Value>
    where
        C: ConfigSpec<Flavor = DomainConfigFlavorPlaintext>,
    {
        Self::assert_compatible::<C>(self).ok()?;
        let stored = self.current_stored_value()?;
        let json = stored.as_plain()?;
        <C::Kind as ValueKind>::decode(json.clone()).ok()
    }

    pub(super) fn current_value_encrypted<C>(
        &self,
        key: &EncryptionKey,
    ) -> Option<<C::Kind as ValueKind>::Value>
    where
        C: ConfigSpec<Flavor = DomainConfigFlavorEncrypted>,
    {
        Self::assert_compatible::<C>(self).ok()?;
        let stored = self.current_stored_value()?;
        let plaintext = stored.decrypt(key).ok()?;
        <C::Kind as ValueKind>::decode(plaintext).ok()
    }

    pub(super) fn update_value_plain<C>(
        &mut self,
        new_value: <C::Kind as ValueKind>::Value,
    ) -> Result<Idempotent<()>, DomainConfigError>
    where
        C: ConfigSpec<Flavor = DomainConfigFlavorPlaintext>,
    {
        Self::assert_compatible::<C>(self)?;
        C::validate(&new_value)?;
        let encoded = <C::Kind as ValueKind>::encode(&new_value)?;
        self.store_value_plain(encoded)
    }

    pub(super) fn update_value_encrypted<C>(
        &mut self,
        key: &EncryptionKey,
        new_value: <C::Kind as ValueKind>::Value,
    ) -> Result<Idempotent<()>, DomainConfigError>
    where
        C: ConfigSpec<Flavor = DomainConfigFlavorEncrypted>,
    {
        Self::assert_compatible::<C>(self)?;
        C::validate(&new_value)?;
        let encoded = <C::Kind as ValueKind>::encode(&new_value)?;
        self.store_value_encrypted(key, encoded)
    }

    pub(super) fn apply_exposed_update_from_json_plain(
        &mut self,
        entry: &crate::registry::ConfigSpecEntry,
        new_value: serde_json::Value,
    ) -> Result<Idempotent<()>, DomainConfigError> {
        self.validate_exposed_update(entry, &new_value)?;
        self.store_value_plain(new_value)
    }

    pub(super) fn apply_exposed_update_from_json_encrypted(
        &mut self,
        entry: &crate::registry::ConfigSpecEntry,
        key: &EncryptionKey,
        new_value: serde_json::Value,
    ) -> Result<Idempotent<()>, DomainConfigError> {
        self.validate_exposed_update(entry, &new_value)?;
        self.store_value_encrypted(key, new_value)
    }

    pub(super) fn apply_update_from_json_plain(
        &mut self,
        entry: &crate::registry::ConfigSpecEntry,
        new_value: serde_json::Value,
    ) -> Result<Idempotent<()>, DomainConfigError> {
        self.validate_update(entry, &new_value)?;
        self.store_value_plain(new_value)
    }

    pub(super) fn apply_update_from_json_encrypted(
        &mut self,
        entry: &crate::registry::ConfigSpecEntry,
        key: &EncryptionKey,
        new_value: serde_json::Value,
    ) -> Result<Idempotent<()>, DomainConfigError> {
        self.validate_update(entry, &new_value)?;
        self.store_value_encrypted(key, new_value)
    }

    fn validate_exposed_update(
        &self,
        entry: &crate::registry::ConfigSpecEntry,
        new_value: &serde_json::Value,
    ) -> Result<(), DomainConfigError> {
        if self.visibility != crate::Visibility::Exposed {
            return Err(DomainConfigError::InvalidState(format!(
                "Config {} is not exposed",
                self.key
            )));
        }
        self.validate_update(entry, new_value)
    }

    fn validate_update(
        &self,
        entry: &crate::registry::ConfigSpecEntry,
        new_value: &serde_json::Value,
    ) -> Result<(), DomainConfigError> {
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

        (entry.validate_json)(new_value)?;

        Ok(())
    }

    fn store_value_plain(
        &mut self,
        plaintext: serde_json::Value,
    ) -> Result<Idempotent<()>, DomainConfigError> {
        // Check idempotency
        if let Some(current) = self.current_stored_value()
            && let Some(current_plain) = current.as_plain()
            && current_plain == &plaintext
        {
            return Ok(Idempotent::AlreadyApplied);
        }

        self.events.push(DomainConfigEvent::Updated {
            value: DomainConfigValue::plain(plaintext),
        });

        Ok(Idempotent::Executed(()))
    }

    fn store_value_encrypted(
        &mut self,
        key: &EncryptionKey,
        plaintext: serde_json::Value,
    ) -> Result<Idempotent<()>, DomainConfigError> {
        // Check idempotency by decrypting and comparing
        if let Some(current) = self.current_stored_value()
            && let Ok(current_plain) = current.decrypt(key)
            && current_plain == plaintext
        {
            return Ok(Idempotent::AlreadyApplied);
        }

        self.events.push(DomainConfigEvent::Updated {
            value: DomainConfigValue::encrypted(key, &plaintext),
        });

        Ok(Idempotent::Executed(()))
    }

    /// Runtime-dispatched version for cases where the config type is not known at compile time.
    /// Uses `self.encrypted` to decide whether to encrypt.
    pub(super) fn apply_exposed_update_from_json_dispatch(
        &mut self,
        entry: &crate::registry::ConfigSpecEntry,
        config: &crate::EncryptionConfig,
        new_value: serde_json::Value,
    ) -> Result<Idempotent<()>, DomainConfigError> {
        if self.encrypted {
            self.apply_exposed_update_from_json_encrypted(entry, &config.key, new_value)
        } else {
            self.apply_exposed_update_from_json_plain(entry, new_value)
        }
    }

    /// Runtime-dispatched version for cases where the config type is not known at compile time.
    /// Uses `self.encrypted` to decide whether to encrypt.
    pub(super) fn apply_update_from_json_dispatch(
        &mut self,
        entry: &crate::registry::ConfigSpecEntry,
        config: &crate::EncryptionConfig,
        new_value: serde_json::Value,
    ) -> Result<Idempotent<()>, DomainConfigError> {
        if self.encrypted {
            self.apply_update_from_json_encrypted(entry, &config.key, new_value)
        } else {
            self.apply_update_from_json_plain(entry, new_value)
        }
    }

    /// Returns the current stored value from the event stream.
    pub fn current_stored_value(&self) -> Option<&DomainConfigValue> {
        self.events.iter_all().rev().find_map(|event| match event {
            DomainConfigEvent::Updated { value } => Some(value),
            _ => None,
        })
    }

    pub(crate) fn assert_compatible<C: ConfigSpec>(entity: &Self) -> Result<(), DomainConfigError> {
        let expected_type = <C::Kind as ValueKind>::TYPE;
        if entity.config_type != expected_type {
            return Err(DomainConfigError::InvalidType(format!(
                "Invalid config type for {key}: expected {expected}, found {found}",
                key = entity.key,
                expected = expected_type,
                found = entity.config_type
            )));
        }

        if entity.visibility != C::VISIBILITY {
            return Err(DomainConfigError::InvalidType(format!(
                "Invalid visibility for {key}: expected {expected}, found {found}",
                key = entity.key,
                expected = C::VISIBILITY,
                found = entity.visibility
            )));
        }

        if entity.encrypted != <C::Flavor as ConfigFlavor>::IS_ENCRYPTED {
            return Err(DomainConfigError::InvalidType(format!(
                "Invalid encrypted flag for {key}: expected {expected}, found {found}",
                key = entity.key,
                expected = <C::Flavor as ConfigFlavor>::IS_ENCRYPTED,
                found = entity.encrypted
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
                    encrypted,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .key(key.clone())
                        .config_type(*config_type)
                        .visibility(*visibility)
                        .encrypted(*encrypted);
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
    pub(super) encrypted: bool,
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
        encrypted: bool,
    ) -> Self {
        self.id(id);
        self.key(key);
        self.config_type(config_type);
        self.visibility(visibility);
        self.encrypted(encrypted);

        self
    }
}

impl IntoEvents<DomainConfigEvent> for NewDomainConfig {
    fn into_events(self) -> EntityEvents<DomainConfigEvent> {
        let events = vec![DomainConfigEvent::Initialized {
            id: self.id,
            key: self.key,
            config_type: self.config_type,
            visibility: self.visibility,
            encrypted: self.encrypted,
        }];

        EntityEvents::init(self.id, events)
    }
}

#[cfg(test)]
mod tests {
    use es_entity::{IntoEvents as _, TryFromEvents as _};
    use serde::{Deserialize, Serialize};

    use super::*;

    fn seed_config<C: ConfigSpec>(id: DomainConfigId) -> DomainConfig {
        let events = NewDomainConfig::builder()
            .seed(
                id,
                C::KEY,
                <C::Kind as ValueKind>::TYPE,
                C::VISIBILITY,
                <C::Flavor as crate::ConfigFlavor>::IS_ENCRYPTED,
            )
            .build()
            .unwrap()
            .into_events();
        DomainConfig::try_from_events(events).unwrap()
    }

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

    crate::define_internal_config! {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
        struct SampleEncryptedConfig {
            secret: String,
        }

        spec {
            key: "encrypted-config";
            encrypted: true;
        }
    }

    #[test]
    fn new_domain_config_emits_expected_init_metadata_for_sample_config() {
        let id = DomainConfigId::new();
        let config = seed_config::<SampleComplexConfig>(id);

        let init = config.events.iter_all().next().expect("init event exists");
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
        let config = seed_config::<SampleSimpleBool>(id);

        let init = config.events.iter_all().next().expect("init event exists");
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
        let mut config = seed_config::<SampleComplexConfig>(id);

        let value = SampleComplexConfig {
            enabled: true,
            limit: 10,
        };

        assert!(
            config
                .update_value_plain::<SampleComplexConfig>(value.clone())
                .unwrap()
                .did_execute()
        );

        let rehydrated_config = DomainConfig::try_from_events(config.events).unwrap();

        assert_eq!(rehydrated_config.id, id);
        assert_eq!(rehydrated_config.key, SampleComplexConfig::KEY);
        assert_eq!(
            rehydrated_config.config_type,
            <<SampleComplexConfig as ConfigSpec>::Kind as ValueKind>::TYPE
        );
        assert_eq!(
            rehydrated_config.visibility,
            SampleComplexConfig::VISIBILITY
        );
        assert_eq!(
            rehydrated_config
                .current_value_plain::<SampleComplexConfig>()
                .unwrap(),
            value
        );
    }

    #[test]
    fn rehydrates_simple_bool_from_events() {
        let id = DomainConfigId::new();
        let mut config = seed_config::<SampleSimpleBool>(id);

        assert!(
            config
                .update_value_plain::<SampleSimpleBool>(false)
                .unwrap()
                .did_execute()
        );

        let rehydrated_config = DomainConfig::try_from_events(config.events).unwrap();

        assert_eq!(rehydrated_config.id, id);
        assert_eq!(rehydrated_config.key, SampleSimpleBool::KEY);
        assert_eq!(
            rehydrated_config.config_type,
            <<SampleSimpleBool as ConfigSpec>::Kind as ValueKind>::TYPE
        );
        assert_eq!(rehydrated_config.visibility, SampleSimpleBool::VISIBILITY);
        assert!(
            !rehydrated_config
                .current_value_plain::<SampleSimpleBool>()
                .unwrap()
        );
    }

    #[test]
    fn update_value_is_idempotent_for_sample_config() {
        let mut config = seed_config::<SampleComplexConfig>(DomainConfigId::new());
        let initial = SampleComplexConfig {
            enabled: true,
            limit: 5,
        };
        let updated = SampleComplexConfig {
            enabled: false,
            limit: 15,
        };

        assert!(
            config
                .update_value_plain::<SampleComplexConfig>(initial)
                .unwrap()
                .did_execute()
        );

        assert!(
            config
                .update_value_plain::<SampleComplexConfig>(updated.clone())
                .expect("first update should succeed")
                .did_execute()
        );

        let result = config
            .update_value_plain::<SampleComplexConfig>(updated.clone())
            .expect("second update should not error");
        assert!(result.was_already_applied());

        let updated_json =
            <<SampleComplexConfig as ConfigSpec>::Kind as ValueKind>::encode(&updated)
                .expect("value encodes");
        let last_event = config.events.iter_all().next_back().unwrap();
        assert!(matches!(
            last_event,
            DomainConfigEvent::Updated { value: DomainConfigValue::Plain { value } } if value == &updated_json
        ));
        assert_eq!(
            config.current_value_plain::<SampleComplexConfig>().unwrap(),
            updated
        );
    }

    #[test]
    fn update_value_is_idempotent_for_simple_bool() {
        let mut config = seed_config::<SampleSimpleBool>(DomainConfigId::new());

        assert!(
            config
                .update_value_plain::<SampleSimpleBool>(false)
                .unwrap()
                .did_execute()
        );

        assert!(
            config
                .update_value_plain::<SampleSimpleBool>(true)
                .expect("first update should succeed")
                .did_execute()
        );

        let result = config
            .update_value_plain::<SampleSimpleBool>(true)
            .expect("second update should not error");
        assert!(result.was_already_applied());

        let updated_json = <<SampleSimpleBool as ConfigSpec>::Kind as ValueKind>::encode(&true)
            .expect("value encodes");
        let last_event = config.events.iter_all().next_back().unwrap();
        assert!(matches!(
            last_event,
            DomainConfigEvent::Updated { value: DomainConfigValue::Plain { value } } if value == &updated_json
        ));
        assert!(config.current_value_plain::<SampleSimpleBool>().unwrap());
    }

    #[test]
    fn update_rejects_invalid_sample_config() {
        let invalid = SampleComplexConfig {
            enabled: true,
            limit: 101,
        };

        let mut config = seed_config::<SampleComplexConfig>(DomainConfigId::new());

        let update_result = config.update_value_plain::<SampleComplexConfig>(invalid);
        assert!(
            matches!(update_result, Err(DomainConfigError::InvalidState(_))),
            "invalid update should fail validation"
        );
    }

    #[test]
    fn current_value_rejects_wrong_type() {
        let config = seed_config::<SampleSimpleBool>(DomainConfigId::new());

        let read_type = config.current_value_plain::<SampleComplexConfig>();
        assert!(read_type.is_none());
    }

    #[test]
    fn current_value_rejects_wrong_visibility() {
        let config = seed_config::<SampleSimpleBool>(DomainConfigId::new());

        let read_visibility = config.current_value_plain::<SampleExposedBool>();
        assert!(read_visibility.is_none());
    }

    #[test]
    fn update_rejects_wrong_type() {
        let mut config = seed_config::<SampleSimpleBool>(DomainConfigId::new());

        let update_type_error =
            config.update_value_plain::<SampleComplexConfig>(SampleComplexConfig {
                enabled: true,
                limit: 1,
            });
        assert!(matches!(
            update_type_error,
            Err(DomainConfigError::InvalidType(message)) if message.contains("config type")
        ));
    }

    #[test]
    fn encrypted_config_roundtrip() {
        let mut config = seed_config::<SampleEncryptedConfig>(DomainConfigId::new());
        assert!(config.encrypted);

        let key = EncryptionKey::default();
        let value = SampleEncryptedConfig {
            secret: "my-secret".to_string(),
        };
        assert!(
            config
                .update_value_encrypted::<SampleEncryptedConfig>(&key, value.clone())
                .unwrap()
                .did_execute()
        );
        assert_eq!(
            config
                .current_value_encrypted::<SampleEncryptedConfig>(&key)
                .unwrap(),
            value
        );

        let stored = config.current_stored_value().unwrap();
        assert!(stored.is_encrypted(), "stored value should be encrypted");
    }

    #[test]
    fn encrypted_config_update_is_idempotent() {
        let mut config = seed_config::<SampleEncryptedConfig>(DomainConfigId::new());

        let key = EncryptionKey::default();
        let value = SampleEncryptedConfig {
            secret: "my-secret".to_string(),
        };
        assert!(
            config
                .update_value_encrypted::<SampleEncryptedConfig>(&key, value.clone())
                .unwrap()
                .did_execute()
        );

        let event_count_after_first = config.events.iter_all().count();

        let result = config
            .update_value_encrypted::<SampleEncryptedConfig>(&key, value)
            .unwrap();
        assert!(result.was_already_applied());
        assert_eq!(config.events.iter_all().count(), event_count_after_first);
    }
}
