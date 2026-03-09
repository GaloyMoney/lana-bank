use derive_builder::Builder;
use es_entity::*;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use encryption::Encrypted;

use crate::{
    ConfigFlavor, ConfigSpec, ConfigType, DomainConfigError, DomainConfigFlavorEncrypted,
    DomainConfigFlavorPlaintext, EncryptionKey, ValueKind,
    error::DomainConfigHydrateError,
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
        default_value: Option<DomainConfigValue>,
    },
    Updated {
        value: DomainConfigValue,
    },
    KeyRotated {
        value: Encrypted,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityHydrationError"))]
pub struct DomainConfig {
    pub id: DomainConfigId,
    pub key: DomainConfigKey,
    pub config_type: ConfigType,
    pub visibility: Visibility,
    pub encrypted: bool,
    events: EntityEvents<DomainConfigEvent>,
}

impl DomainConfig {
    /// Asserts that the entity's current business value (if any) is readable
    /// with the given encryption key (or the deprecated key, when provided).
    ///
    /// The check is tolerant of key rotations that haven't yet propagated to
    /// this runtime instance: a `KeyRotated` event written with a *different*
    /// key is fine as long as no *new* `Updated` event was written with that
    /// key afterwards.  Once a real business update happens with a key we
    /// cannot read, this instance's view of the value is stale and hydration
    /// must fail.
    ///
    /// `deprecated_key` is needed during key-rotation flows so that configs
    /// still encrypted with the old key pass hydration while they are being
    /// re-encrypted.
    pub(crate) fn assert_decryptable(
        &self,
        key: &EncryptionKey,
        deprecated_key: Option<&EncryptionKey>,
    ) -> Result<(), DomainConfigHydrateError> {
        if !self.encrypted {
            return Ok(());
        }

        // Walk the event stream backwards to find the most recent
        // value-carrying event and verify we can read it.
        for event in self.events.iter_all().rev() {
            match event {
                DomainConfigEvent::Updated { value } => {
                    if !value.is_encrypted() || value.matches_any_key(key, deprecated_key) {
                        return Ok(());
                    }
                    // An `Updated` event we cannot read — a real business
                    // value was written with a key we don't have.  No older
                    // event can change this.
                    return Err(DomainConfigHydrateError::new(
                        self.key.clone(),
                        "encrypted value cannot be decrypted with the current runtime key",
                    ));
                }
                DomainConfigEvent::KeyRotated { value } => {
                    if value.matches_key(key)
                        || deprecated_key.is_some_and(|dk| value.matches_key(dk))
                    {
                        // A re-encryption we can read.  `KeyRotated` events
                        // carry the same business value, so this confirms
                        // readability.
                        return Ok(());
                    }
                    // KeyRotated with a different key — skip and look for
                    // an older event we can read.
                }
                DomainConfigEvent::Initialized { .. } => {}
            }
        }

        // No encrypted values stored at all (no Updated events, or only
        // a plaintext default in Initialized).
        Ok(())
    }

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
        let stored = self.current_stored_value_for_key(key)?;
        let plaintext = stored.decrypt(key).ok()?;
        <C::Kind as ValueKind>::decode(plaintext).ok()
    }

    fn current_stored_value_for_key(&self, key: &EncryptionKey) -> Option<DomainConfigValue> {
        self.events.iter_all().rev().find_map(|event| match event {
            DomainConfigEvent::KeyRotated { value } if value.matches_key(key) => {
                Some(DomainConfigValue::Encrypted(value.clone()))
            }
            DomainConfigEvent::Updated { value } if value.matches_key(key) => Some(value.clone()),
            _ => None,
        })
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

        if self.encrypted != entry.encrypted {
            return Err(DomainConfigError::InvalidType(format!(
                "Invalid encrypted flag for {}: expected {}, found {}",
                self.key, self.encrypted, entry.encrypted
            )));
        }

        (entry.validate_json)(new_value)?;

        Ok(())
    }

    fn store_value_plain(
        &mut self,
        plaintext: serde_json::Value,
    ) -> Result<Idempotent<()>, DomainConfigError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            already_applied: DomainConfigEvent::Updated { value } if value.as_plain() == Some(&plaintext),
            resets_on: DomainConfigEvent::Updated { .. }
        );

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
        idempotency_guard!(
            self.events.iter_all().rev(),
            already_applied: DomainConfigEvent::Updated { value }
                if value.matches_key(key)
                    && value.decrypt(key).ok().as_ref() == Some(&plaintext),
            already_applied: DomainConfigEvent::KeyRotated { value }
                if value.matches_key(key)
                    && key.decrypt_json::<serde_json::Value>(value).ok().as_ref() == Some(&plaintext),
            resets_on: DomainConfigEvent::Updated { .. } | DomainConfigEvent::KeyRotated { .. }
        );

        // Block writes if the latest event has a different key (has been rotated)
        if let Some(latest) = self.current_stored_value()
            && !latest.matches_key(key)
        {
            return Err(DomainConfigError::StaleEncryptionKey);
        }

        self.events.push(DomainConfigEvent::Updated {
            value: DomainConfigValue::encrypted(key, &plaintext),
        });

        Ok(Idempotent::Executed(()))
    }

    /// Runtime-dispatched version for cases where the config type is not known at compile time.
    /// Uses `self.encrypted` to decide whether to encrypt.
    pub(super) fn apply_exposed_update_from_json(
        &mut self,
        entry: &crate::registry::ConfigSpecEntry,
        config: &crate::EncryptionConfig,
        new_value: serde_json::Value,
    ) -> Result<Idempotent<()>, DomainConfigError> {
        self.validate_exposed_update(entry, &new_value)?;
        if self.encrypted {
            self.store_value_encrypted(&config.encryption_key, new_value)
        } else {
            self.store_value_plain(new_value)
        }
    }

    /// Runtime-dispatched version for cases where the config type is not known at compile time.
    /// Uses `self.encrypted` to decide whether to encrypt.
    pub(super) fn apply_update_from_json(
        &mut self,
        entry: &crate::registry::ConfigSpecEntry,
        config: &crate::EncryptionConfig,
        new_value: serde_json::Value,
    ) -> Result<Idempotent<()>, DomainConfigError> {
        self.validate_update(entry, &new_value)?;
        if self.encrypted {
            self.store_value_encrypted(&config.encryption_key, new_value)
        } else {
            self.store_value_plain(new_value)
        }
    }

    /// Returns the current stored value from the event stream,
    /// falling back to the default from the Initialized event if nothing is stored.
    pub fn current_stored_value(&self) -> Option<DomainConfigValue> {
        self.events.iter_all().rev().find_map(|event| match event {
            DomainConfigEvent::KeyRotated { value } => {
                Some(DomainConfigValue::Encrypted(value.clone()))
            }
            DomainConfigEvent::Updated { value } => Some(value.clone()),
            DomainConfigEvent::Initialized { default_value, .. } => default_value.clone(),
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

    pub(super) fn rotate_encryption_key(
        &mut self,
        new_key: &EncryptionKey,
        deprecated_key: &EncryptionKey,
    ) -> Result<Idempotent<()>, DomainConfigError> {
        if !self.encrypted {
            return Err(DomainConfigError::NotEncrypted(
                "Cannot perform key rotation for a non-encrypted config".to_string(),
            ));
        }

        let Some(current) = self.current_stored_value() else {
            return Ok(Idempotent::AlreadyApplied);
        };
        if current.matches_key(new_key) {
            return Ok(Idempotent::AlreadyApplied);
        }

        let new_encrypted = current.rotate(new_key, deprecated_key)?;
        self.events.push(DomainConfigEvent::KeyRotated {
            value: new_encrypted,
        });

        Ok(Idempotent::Executed(()))
    }
}

impl TryFromEvents<DomainConfigEvent> for DomainConfig {
    fn try_from_events(
        events: EntityEvents<DomainConfigEvent>,
    ) -> Result<Self, EntityHydrationError> {
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
                DomainConfigEvent::KeyRotated { .. } => {}
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
    #[builder(default)]
    pub(super) default_value: Option<DomainConfigValue>,
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
            default_value: self.default_value,
        }];

        EntityEvents::init(self.id, events)
    }
}

#[cfg(test)]
mod tests {
    use es_entity::{IntoEvents as _, TryFromEvents as _};
    use serde::{Deserialize, Serialize};

    use super::*;

    use crate::EncryptionConfig;

    fn seed_config<C: ConfigSpec>(id: DomainConfigId) -> DomainConfig {
        let default_value = C::default_value()
            .and_then(|v| <C::Kind as ValueKind>::encode(&v).ok())
            .map(DomainConfigValue::plain);

        let events = NewDomainConfig::builder()
            .seed(
                id,
                C::KEY,
                <C::Kind as ValueKind>::TYPE,
                C::VISIBILITY,
                <C::Flavor as crate::ConfigFlavor>::IS_ENCRYPTED,
            )
            .default_value(default_value)
            .build()
            .unwrap()
            .into_events();
        DomainConfig::try_from_events(events).unwrap()
    }

    fn encryption_config() -> EncryptionConfig {
        EncryptionConfig::default()
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
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
        struct SampleComplexWithDefaultConfig {
            enabled: bool,
            limit: u32,
        }

        spec {
            key: "sample-config-with-default";
            default: || Some(SampleComplexWithDefaultConfig{
                enabled: false,
                limit:100
            });
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

    #[test]
    fn update_via_json_rejects_wrong_encrypted_flag() {
        let mut config = seed_config::<SampleEncryptedConfig>(DomainConfigId::new());

        let value = serde_json::to_value(&SampleEncryptedConfig {
            secret: "my-secret".to_string(),
        })
        .unwrap();

        let entry = crate::registry::maybe_find_by_key("encrypted-config").unwrap();
        let mut wrong_entry = *entry;
        wrong_entry.encrypted = false;

        let encryption_config = encryption_config();
        let result = config.apply_update_from_json(&wrong_entry, &encryption_config, value);
        assert!(matches!(
            result,
            Err(DomainConfigError::InvalidType(message)) if message.contains("encrypted flag")
        ));
    }

    #[test]
    fn key_rotation_is_idempotent() {
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

        let new_key = EncryptionKey::new([1; 32]);

        assert!(
            config
                .rotate_encryption_key(&new_key, &key)
                .unwrap()
                .did_execute()
        );

        assert!(
            config
                .rotate_encryption_key(&new_key, &key)
                .unwrap()
                .was_already_applied()
        );

        assert_eq!(
            config
                .current_value_encrypted::<SampleEncryptedConfig>(&new_key)
                .unwrap(),
            value
        );
    }

    // -----------------------------------------------------------------------
    // assert_decryptable (post-hydrate hook validation)
    // -----------------------------------------------------------------------

    #[test]
    fn assert_decryptable_passes_for_non_encrypted_config() {
        let config = seed_config::<SampleComplexConfig>(DomainConfigId::new());
        assert!(!config.encrypted);
        assert!(
            config
                .assert_decryptable(&EncryptionKey::default(), None)
                .is_ok()
        );
    }

    #[test]
    fn assert_decryptable_passes_when_no_value_stored() {
        let config = seed_config::<SampleEncryptedConfig>(DomainConfigId::new());
        assert!(config.encrypted);
        // No Updated events yet — only Initialized with no default.
        assert!(
            config
                .assert_decryptable(&EncryptionKey::default(), None)
                .is_ok()
        );
    }

    #[test]
    fn assert_decryptable_passes_when_value_matches_key() {
        let mut config = seed_config::<SampleEncryptedConfig>(DomainConfigId::new());
        let key = EncryptionKey::default();
        let _ = config
            .update_value_encrypted::<SampleEncryptedConfig>(
                &key,
                SampleEncryptedConfig { secret: "s".into() },
            )
            .unwrap();

        assert!(config.assert_decryptable(&key, None).is_ok());
    }

    #[test]
    fn assert_decryptable_passes_after_key_rotation_with_old_key() {
        // Updated(old) → KeyRotated(new)
        // Old-key instance can still read from the Updated event.
        let mut config = seed_config::<SampleEncryptedConfig>(DomainConfigId::new());
        let old_key = EncryptionKey::default();
        let new_key = EncryptionKey::new([1; 32]);

        let _ = config
            .update_value_encrypted::<SampleEncryptedConfig>(
                &old_key,
                SampleEncryptedConfig { secret: "s".into() },
            )
            .unwrap();
        let _ = config.rotate_encryption_key(&new_key, &old_key).unwrap();

        // Old key should still pass — no new business update after rotation.
        assert!(config.assert_decryptable(&old_key, None).is_ok());
        // New key should also pass.
        assert!(config.assert_decryptable(&new_key, None).is_ok());
    }

    #[test]
    fn assert_decryptable_fails_after_update_with_new_key() {
        // Updated(old) → KeyRotated(new) → Updated(new)
        // Old-key instance cannot read the latest business value.
        let mut config = seed_config::<SampleEncryptedConfig>(DomainConfigId::new());
        let old_key = EncryptionKey::default();
        let new_key = EncryptionKey::new([1; 32]);

        let _ = config
            .update_value_encrypted::<SampleEncryptedConfig>(
                &old_key,
                SampleEncryptedConfig { secret: "s".into() },
            )
            .unwrap();
        let _ = config.rotate_encryption_key(&new_key, &old_key).unwrap();
        let _ = config
            .update_value_encrypted::<SampleEncryptedConfig>(
                &new_key,
                SampleEncryptedConfig {
                    secret: "new-secret".into(),
                },
            )
            .unwrap();

        // Old key alone should fail — there's a newer Updated event it can't read.
        assert!(config.assert_decryptable(&old_key, None).is_err());
        // New key should pass.
        assert!(config.assert_decryptable(&new_key, None).is_ok());
    }

    #[test]
    fn assert_decryptable_passes_with_deprecated_key() {
        // Updated(old) → KeyRotated(new) → Updated(new)
        // When the old key is provided as deprecated_key alongside the new
        // key, all events are readable — should pass.
        let mut config = seed_config::<SampleEncryptedConfig>(DomainConfigId::new());
        let old_key = EncryptionKey::default();
        let new_key = EncryptionKey::new([1; 32]);

        let _ = config
            .update_value_encrypted::<SampleEncryptedConfig>(
                &old_key,
                SampleEncryptedConfig { secret: "s".into() },
            )
            .unwrap();
        let _ = config.rotate_encryption_key(&new_key, &old_key).unwrap();
        let _ = config
            .update_value_encrypted::<SampleEncryptedConfig>(
                &new_key,
                SampleEncryptedConfig {
                    secret: "new-secret".into(),
                },
            )
            .unwrap();

        // With deprecated key, the new key can read Updated(new) directly.
        assert!(config.assert_decryptable(&new_key, Some(&old_key)).is_ok());
    }

    #[test]
    fn assert_decryptable_passes_with_deprecated_key_before_rotation() {
        // Updated(old) only — value was written with old key, which is now
        // the deprecated key.  The new primary key can't read it, but the
        // deprecated key can.
        let mut config = seed_config::<SampleEncryptedConfig>(DomainConfigId::new());
        let old_key = EncryptionKey::default();
        let new_key = EncryptionKey::new([1; 32]);

        let _ = config
            .update_value_encrypted::<SampleEncryptedConfig>(
                &old_key,
                SampleEncryptedConfig { secret: "s".into() },
            )
            .unwrap();

        // new_key alone can't read it.
        assert!(config.assert_decryptable(&new_key, None).is_err());
        // But with the old key as deprecated, it passes.
        assert!(config.assert_decryptable(&new_key, Some(&old_key)).is_ok());
    }

    #[test]
    fn assert_decryptable_fails_with_completely_unknown_key() {
        let mut config = seed_config::<SampleEncryptedConfig>(DomainConfigId::new());
        let key = EncryptionKey::default();
        let unknown_key = EncryptionKey::new([99; 32]);

        let _ = config
            .update_value_encrypted::<SampleEncryptedConfig>(
                &key,
                SampleEncryptedConfig { secret: "s".into() },
            )
            .unwrap();

        assert!(config.assert_decryptable(&unknown_key, None).is_err());
    }

    #[test]
    fn current_stored_value_returns_default_when_no_update() {
        let config = seed_config::<SampleComplexWithDefaultConfig>(DomainConfigId::new());
        assert_eq!(
            config
                .current_value_plain::<SampleComplexWithDefaultConfig>()
                .unwrap(),
            SampleComplexWithDefaultConfig::default_value().unwrap(),
        );
    }

    #[test]
    fn current_stored_value_prefers_update_over_default() {
        let mut config = seed_config::<SampleComplexWithDefaultConfig>(DomainConfigId::new());

        let updated = SampleComplexWithDefaultConfig {
            enabled: true,
            limit: 50,
        };
        assert!(
            config
                .update_value_plain::<SampleComplexWithDefaultConfig>(updated.clone())
                .unwrap()
                .did_execute()
        );

        let expected = serde_json::to_value(updated).unwrap();
        assert_eq!(
            config.current_stored_value().unwrap().as_plain(),
            Some(&expected)
        );
    }
}
