#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{borrow::Cow, str::FromStr};

use crate::DomainConfigError;

es_entity::entity_id! {
    DomainConfigId,
}

pub trait DomainConfigValue: Serialize + DeserializeOwned + Clone {
    const KEY: DomainConfigKey;

    fn validate(&self) -> Result<(), DomainConfigError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct DomainConfigKey(Cow<'static, str>);

impl DomainConfigKey {
    pub const fn new(key: &'static str) -> Self {
        DomainConfigKey(Cow::Borrowed(key))
    }

    fn from_owned(key: String) -> Self {
        DomainConfigKey(Cow::Owned(key))
    }
}

impl std::fmt::Display for DomainConfigKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for DomainConfigKey {
    fn from(value: String) -> Self {
        DomainConfigKey::from_owned(value)
    }
}

impl From<&str> for DomainConfigKey {
    fn from(value: &str) -> Self {
        DomainConfigKey::from_owned(value.to_owned())
    }
}

impl FromStr for DomainConfigKey {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(DomainConfigKey::from_owned(s.to_owned()))
    }
}
