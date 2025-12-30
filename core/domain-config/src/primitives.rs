#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt, str::FromStr};

es_entity::entity_id! {
    DomainConfigId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "lowercase")]
pub enum ConfigType {
    Bool,
    String,
    Int,
    Uint,
    Decimal,
    Complex,
}

impl ConfigType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfigType::Bool => "bool",
            ConfigType::String => "string",
            ConfigType::Int => "int",
            ConfigType::Uint => "uint",
            ConfigType::Decimal => "decimal",
            ConfigType::Complex => "complex",
        }
    }

    pub fn is_simple(&self) -> bool {
        !matches!(self, ConfigType::Complex)
    }
}

impl fmt::Display for ConfigType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "lowercase")]
pub enum Visibility {
    Exposed,
    Internal,
}

impl Visibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Visibility::Exposed => "exposed",
            Visibility::Internal => "internal",
        }
    }
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
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
