use std::marker::PhantomData;

use encryption::KeyId;

use crate::{
    ConfigSpec, DefaultedConfig, DomainConfigError, DomainConfigFlavorEncrypted,
    DomainConfigFlavorPlaintext, EncryptionKey, ValueKind, entity::DomainConfig,
    flavor::ConfigFlavor,
};

pub struct TypedDomainConfig<C: ConfigSpec> {
    pub(crate) entity: DomainConfig,
    pub(crate) _marker: PhantomData<C>,
    pub(crate) encryption_key: Option<EncryptionKey>,
    pub(crate) encryption_key_id: Option<KeyId>,
}

impl<C: ConfigSpec<Flavor = DomainConfigFlavorPlaintext>> TypedDomainConfig<C> {
    pub(crate) fn try_new_plain(entity: DomainConfig) -> Result<Self, DomainConfigError> {
        DomainConfig::assert_compatible::<C>(&entity)?;
        Ok(Self {
            entity,
            _marker: PhantomData,
            encryption_key: None,
            encryption_key_id: None,
        })
    }
}

impl<C: ConfigSpec<Flavor = DomainConfigFlavorEncrypted>> TypedDomainConfig<C> {
    pub(crate) fn try_new_encrypted(
        entity: DomainConfig,
        key: EncryptionKey,
        key_id: KeyId,
    ) -> Result<Self, DomainConfigError> {
        DomainConfig::assert_compatible::<C>(&entity)?;
        Ok(Self {
            entity,
            _marker: PhantomData,
            encryption_key: Some(key),
            encryption_key_id: Some(key_id),
        })
    }
}

impl<C: ConfigSpec> TypedDomainConfig<C> {
    /// Returns the config value as `Option<T>`.
    ///
    /// Use this for configs without compile-time defaults, or when you need
    /// to distinguish between "not set" and "set to default".
    pub fn maybe_value(&self) -> Option<<C::Kind as ValueKind>::Value> {
        C::Flavor::maybe_value(self)
    }
}

impl<C: DefaultedConfig> TypedDomainConfig<C> {
    /// Returns the config value directly.
    ///
    /// This method is only available for configs with compile-time defaults
    /// (those defined with a `default:` clause). The value is guaranteed to
    /// exist because the default is always available.
    pub fn value(&self) -> <C::Kind as ValueKind>::Value {
        C::Flavor::maybe_value(self).expect("DefaultedConfig guarantees a value")
    }
}
