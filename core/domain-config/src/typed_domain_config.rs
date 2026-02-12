use std::marker::PhantomData;

use crate::{
    ConfigSpec, ConfigType, DefaultedConfig, DomainConfigError, DomainConfigFlavorEncrypted,
    DomainConfigFlavorPlaintext, DomainConfigKey, ValueKind, Visibility,
    encryption::EncryptionKey,
    flavor::FlavorDispatch,
};

use crate::entity::DomainConfig;

pub struct TypedDomainConfig<C: ConfigSpec> {
    entity: DomainConfig,
    _marker: PhantomData<C>,
    encryption_key: Option<EncryptionKey>,
}

impl<C: ConfigSpec> TypedDomainConfig<C> {
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
