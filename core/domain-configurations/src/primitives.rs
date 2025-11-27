use authz::{ActionPermission, AllOrOne};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub const PERMISSION_SET_DOMAIN_CONFIGURATION_VIEWER: &str = "domain-configuration-viewer";
pub const PERMISSION_SET_DOMAIN_CONFIGURATION_WRITER: &str = "domain-configuration-writer";

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString, strum::VariantArray,
)]
#[strum(serialize_all = "kebab-case")]
pub enum DomainConfigurationAction {
    Read,
    Write,
}

impl ActionPermission for DomainConfigurationAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read => PERMISSION_SET_DOMAIN_CONFIGURATION_VIEWER,
            Self::Write => PERMISSION_SET_DOMAIN_CONFIGURATION_WRITER,
        }
    }
}

impl DomainConfigurationAction {
    pub fn actions() -> Vec<authz::action_description::ActionMapping> {
        use authz::action_description::ActionMapping;
        Self::VARIANTS
            .iter()
            .map(|variant| {
                ActionMapping::new(
                    "domain-configuration",
                    "domain-configuration",
                    variant,
                    variant.permission_set(),
                )
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct DomainConfigurationKey(String);

impl DomainConfigurationKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DomainConfigurationKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for DomainConfigurationKey {
    fn from(value: String) -> Self {
        DomainConfigurationKey::new(value)
    }
}

impl From<&str> for DomainConfigurationKey {
    fn from(value: &str) -> Self {
        DomainConfigurationKey::new(value.to_owned())
    }
}

impl FromStr for DomainConfigurationKey {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(DomainConfigurationKey::new(s.to_owned()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum DomainConfigurationObject {
    DomainConfiguration(AllOrOne<DomainConfigurationKey>),
}

impl DomainConfigurationObject {
    pub fn all() -> Self {
        Self::DomainConfiguration(AllOrOne::All)
    }

    pub fn key(key: DomainConfigurationKey) -> Self {
        Self::DomainConfiguration(AllOrOne::ById(key))
    }
}

impl std::fmt::Display for DomainConfigurationObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DomainConfigurationObject::*;
        match self {
            DomainConfiguration(obj) => write!(f, "domain-configuration/{obj}"),
        }
    }
}

impl FromStr for DomainConfigurationObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, rest) = s
            .split_once('/')
            .ok_or("missing separator in DomainConfigurationObject")?;
        if entity != "domain-configuration" {
            return Err("invalid entity in DomainConfigurationObject");
        }
        let key = rest
            .parse::<AllOrOne<DomainConfigurationKey>>()
            .map_err(|_| "invalid key in DomainConfigurationObject")?;
        Ok(DomainConfigurationObject::DomainConfiguration(key))
    }
}

pub trait ConfigKey<T> {
    fn key() -> DomainConfigurationKey;

    fn object() -> DomainConfigurationObject {
        DomainConfigurationObject::key(Self::key())
    }

    fn required_action_read() -> DomainConfigurationAction {
        DomainConfigurationAction::Read
    }

    fn required_action_write() -> DomainConfigurationAction {
        DomainConfigurationAction::Write
    }
}
