use std::marker::PhantomData;

use crate::{
    DomainConfigValue,
    simple::{SimpleConfig, SimpleScalar, SimpleType},
    DomainConfigKey,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigKind {
    Simple(SimpleType),
    Complex,
}

pub trait ConfigSpec: sealed::Sealed {
    type Value;
    fn kind() -> ConfigKind;
}

/// Handle for structured domain configs backed by a `DomainConfigValue`.
#[derive(Debug, Clone, Copy)]
pub struct TypedConfig<T: DomainConfigValue> {
    _pd: PhantomData<T>,
}

impl<T: DomainConfigValue> TypedConfig<T> {
    pub const fn new() -> Self {
        Self {
            _pd: PhantomData,
        }
    }

    pub const fn key(&self) -> DomainConfigKey {
        T::KEY
    }
}

impl<T: DomainConfigValue> ConfigSpec for TypedConfig<T> {
    type Value = T;

    fn kind() -> ConfigKind {
        ConfigKind::Complex
    }
}

impl<T: SimpleScalar> ConfigSpec for SimpleConfig<T> {
    type Value = T;

    fn kind() -> ConfigKind {
        ConfigKind::Simple(T::SIMPLE_TYPE)
    }
}

pub(crate) mod sealed {
    pub trait Sealed {}
    impl<T: crate::DomainConfigValue> Sealed for super::TypedConfig<T> {}
    impl<T: crate::SimpleScalar> Sealed for crate::SimpleConfig<T> {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use crate::{DomainConfigError, DomainConfigKey, DomainConfigValue, SimpleType};

    #[derive(Clone, Default, Serialize, Deserialize)]
    struct DummyConfig;

    impl DomainConfigValue for DummyConfig {
        const KEY: DomainConfigKey = DomainConfigKey::new("dummy-config");

        fn validate(&self) -> Result<(), DomainConfigError> {
            Ok(())
        }
    }

    #[test]
    fn specs_encode_kind_and_key() {
        const SIMPLE: SimpleConfig<bool> = SimpleConfig::new("feature_x_enabled");
        const COMPLEX: TypedConfig<DummyConfig> = TypedConfig::new();

        assert!(matches!(
            SimpleConfig::<bool>::kind(),
            ConfigKind::Simple(SimpleType::Bool)
        ));
        assert!(matches!(TypedConfig::<DummyConfig>::kind(), ConfigKind::Complex));

        let _: &'static str = SIMPLE.key;
        let _: DomainConfigKey = COMPLEX.key();
    }
}
