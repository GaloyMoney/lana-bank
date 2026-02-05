use std::marker::PhantomData;

use crate::{
    ConfigSpec, ConfigType, DefaultedConfig, DomainConfigError, DomainConfigKey, ValueKind,
    Visibility,
};

use crate::entity::DomainConfig;

pub struct TypedDomainConfig<C: ConfigSpec> {
    entity: DomainConfig,
    _marker: PhantomData<C>,
}

impl<C: ConfigSpec> TypedDomainConfig<C> {
    pub(crate) fn new(entity: DomainConfig) -> Result<Self, DomainConfigError> {
        DomainConfig::assert_compatible::<C>(&entity)?;
        Ok(Self {
            entity,
            _marker: PhantomData,
        })
    }

    /// Returns the config value as `Option<T>`.
    ///
    /// Use this for configs without compile-time defaults, or when you need
    /// to distinguish between "not set" and "set to default".
    pub fn maybe_value(&self) -> Option<<C::Kind as ValueKind>::Value> {
        self.entity.current_value::<C>().or_else(C::default_value)
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

impl<C: DefaultedConfig> TypedDomainConfig<C> {
    /// Returns the config value directly.
    ///
    /// This method is only available for configs with compile-time defaults
    /// (those defined with a `default:` clause). The value is guaranteed to
    /// exist because the default is always available.
    pub fn value(&self) -> <C::Kind as ValueKind>::Value {
        self.maybe_value()
            .expect("DefaultedConfig guarantees a value")
    }
}
