use std::marker::PhantomData;

use crate::{
    ConfigSpec, ConfigType, DefaultedConfig, DomainConfigError, DomainConfigFlavorEncrypted,
    DomainConfigFlavorPlaintext, DomainConfigKey, ValueKind, Visibility,
    encryption::{EncryptionKey, StorageEncryption},
};

use crate::entity::DomainConfig;

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::DomainConfigFlavorPlaintext {}
    impl Sealed for super::DomainConfigFlavorEncrypted {}
}

/// Trait for compile-time dispatch based on config flavor.
/// This allows unified `maybe_value()` and `value()` methods on TypedDomainConfig.
pub trait FlavorDispatch: sealed::Sealed {
    fn maybe_value<C: ConfigSpec<Flavor = Self>>(
        config: &TypedDomainConfig<C>,
    ) -> Option<<C::Kind as ValueKind>::Value>;
}

impl FlavorDispatch for DomainConfigFlavorPlaintext {
    fn maybe_value<C: ConfigSpec<Flavor = Self>>(
        config: &TypedDomainConfig<C>,
    ) -> Option<<C::Kind as ValueKind>::Value> {
        config.maybe_value_plain()
    }
}

impl FlavorDispatch for DomainConfigFlavorEncrypted {
    fn maybe_value<C: ConfigSpec<Flavor = Self>>(
        config: &TypedDomainConfig<C>,
    ) -> Option<<C::Kind as ValueKind>::Value> {
        config.maybe_value_encrypted()
    }
}

pub struct TypedDomainConfig<C: ConfigSpec> {
    entity: DomainConfig,
    _marker: PhantomData<C>,
    encryption_key: Option<EncryptionKey>,
    // TODO: remove once migration to flavor-based methods is complete
    encryption_old: Option<StorageEncryption>,
}

impl<C: ConfigSpec> TypedDomainConfig<C> {
    /// Deprecated: use flavor-specific try_new instead
    pub(crate) fn try_new_old(
        entity: DomainConfig,
        encryption: StorageEncryption,
    ) -> Result<Self, DomainConfigError> {
        DomainConfig::assert_compatible::<C>(&entity)?;
        Ok(Self {
            entity,
            _marker: PhantomData,
            encryption_key: None,
            encryption_old: Some(encryption),
        })
    }

    /// Deprecated: use flavor-specific maybe_value instead
    pub fn maybe_value_old(&self) -> Option<<C::Kind as ValueKind>::Value> {
        let encryption = self.encryption_old.as_ref()?;
        self.entity
            .current_value::<C>(encryption)
            .or_else(C::default_value)
    }

    pub fn default_value(&self) -> Option<<C::Kind as ValueKind>::Value> {
        C::default_value()
    }

    pub fn key(&self) -> DomainConfigKey {
        self.entity.key.clone()
    }

    pub fn visibility(&self) -> Visibility {
        self.entity.visibility
    }

    pub fn config_type(&self) -> ConfigType {
        self.entity.config_type
    }
}

impl<C: ConfigSpec<Flavor = DomainConfigFlavorPlaintext>> TypedDomainConfig<C> {
    pub(crate) fn try_new_plain(entity: DomainConfig) -> Result<Self, DomainConfigError> {
        DomainConfig::assert_compatible::<C>(&entity)?;
        Ok(Self {
            entity,
            _marker: PhantomData,
            encryption_key: None,
            encryption_old: None,
        })
    }

    /// Returns the config value as `Option<T>`.
    ///
    /// Use this for configs without compile-time defaults, or when you need
    /// to distinguish between "not set" and "set to default".
    pub fn maybe_value_plain(&self) -> Option<<C::Kind as ValueKind>::Value> {
        self.entity
            .current_value_plain::<C>()
            .or_else(C::default_value)
    }
}

impl<C: ConfigSpec<Flavor = DomainConfigFlavorEncrypted>> TypedDomainConfig<C> {
    pub(crate) fn try_new_encrypted(
        entity: DomainConfig,
        key: EncryptionKey,
    ) -> Result<Self, DomainConfigError> {
        DomainConfig::assert_compatible::<C>(&entity)?;
        Ok(Self {
            entity,
            _marker: PhantomData,
            encryption_key: Some(key),
            encryption_old: None,
        })
    }

    /// Returns the config value as `Option<T>`.
    ///
    /// Use this for configs without compile-time defaults, or when you need
    /// to distinguish between "not set" and "set to default".
    pub fn maybe_value_encrypted(&self) -> Option<<C::Kind as ValueKind>::Value> {
        let key = self.encryption_key.as_ref()?;
        self.entity
            .current_value_encrypted::<C>(key)
            .or_else(C::default_value)
    }
}

impl<C: ConfigSpec> TypedDomainConfig<C>
where
    C::Flavor: FlavorDispatch,
{
    /// Returns the config value as `Option<T>`.
    ///
    /// Use this for configs without compile-time defaults, or when you need
    /// to distinguish between "not set" and "set to default".
    pub fn maybe_value(&self) -> Option<<C::Kind as ValueKind>::Value> {
        C::Flavor::maybe_value(self)
    }
}

impl<C: DefaultedConfig> TypedDomainConfig<C>
where
    C::Flavor: FlavorDispatch,
{
    /// Returns the config value directly.
    ///
    /// This method is only available for configs with compile-time defaults
    /// (those defined with a `default:` clause). The value is guaranteed to
    /// exist because the default is always available.
    pub fn value(&self) -> <C::Kind as ValueKind>::Value {
        C::Flavor::maybe_value(self).expect("DefaultedConfig guarantees a value")
    }
}
