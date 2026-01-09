use std::marker::PhantomData;

use crate::{ConfigSpec, ConfigType, DomainConfigError, DomainConfigKey, ValueKind, Visibility};

use crate::entity::DomainConfig;

pub struct ConfigView<C: ConfigSpec> {
    entity: DomainConfig,
    _marker: PhantomData<C>,
}

impl<C: ConfigSpec> ConfigView<C> {
    pub(crate) fn new(entity: DomainConfig) -> Result<Self, DomainConfigError> {
        entity.ensure::<C>()?;
        Ok(Self {
            entity,
            _marker: PhantomData,
        })
    }

    pub fn is_configured(&self) -> bool {
        self.entity.is_configured()
    }

    pub fn value(&self) -> Result<<C::Kind as ValueKind>::Value, DomainConfigError> {
        self.entity.current_value::<C>()
    }

    pub fn value_or_default(&self) -> Result<<C::Kind as ValueKind>::Value, DomainConfigError> {
        if self.is_configured() {
            self.value()
        } else {
            C::default_value().ok_or_else(|| DomainConfigError::NoDefault(C::KEY.to_string()))
        }
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
