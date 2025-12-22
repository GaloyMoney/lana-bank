use serde::{Serialize, de::DeserializeOwned};

use crate::{DomainConfigError, primitives::DomainConfigKey};

pub trait ComplexConfig: Serialize + DeserializeOwned + Clone + Default {
    const KEY: DomainConfigKey;

    fn validate(&self) -> Result<(), DomainConfigError>;
}
