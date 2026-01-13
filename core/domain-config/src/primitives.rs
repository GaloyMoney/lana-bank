pub use authz::{ActionPermission, AllOrOne, action_description::*, map_action};
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

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
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

pub const PERMISSION_SET_EXPOSED_CONFIGS_VIEWER: &str = "exposed_configs_viewer";
pub const PERMISSION_SET_EXPOSED_CONFIGS_WRITER: &str = "exposed_configs_writer";

pub type ExposedConfigAllOrOne = AllOrOne<DomainConfigId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum DomainConfigObject {
    ExposedConfig(ExposedConfigAllOrOne),
}

impl DomainConfigObject {
    pub const fn all_exposed_configs() -> Self {
        Self::ExposedConfig(AllOrOne::All)
    }
}

impl fmt::Display for DomainConfigObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let discriminant = DomainConfigObjectDiscriminants::from(self);
        match self {
            Self::ExposedConfig(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
        }
    }
}

impl FromStr for DomainConfigObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use DomainConfigObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            ExposedConfig => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse DomainConfigObject")?;
                Self::ExposedConfig(obj_ref)
            }
        };
        Ok(res)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum DomainConfigAction {
    ExposedConfig(DomainConfigEntityAction),
}

impl DomainConfigAction {
    pub const EXPOSED_CONFIG_READ: Self =
        DomainConfigAction::ExposedConfig(DomainConfigEntityAction::Read);
    pub const EXPOSED_CONFIG_WRITE: Self =
        DomainConfigAction::ExposedConfig(DomainConfigEntityAction::Write);

    pub fn actions() -> Vec<ActionMapping> {
        use DomainConfigActionDiscriminants::*;
        map_action!(domain_config, ExposedConfig, DomainConfigEntityAction)
    }
}

impl fmt::Display for DomainConfigAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:", DomainConfigActionDiscriminants::from(self))?;
        match self {
            DomainConfigAction::ExposedConfig(action) => action.fmt(f),
        }
    }
}

impl FromStr for DomainConfigAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use DomainConfigActionDiscriminants::*;
        let res = match entity.parse()? {
            ExposedConfig => DomainConfigAction::from(action.parse::<DomainConfigEntityAction>()?),
        };
        Ok(res)
    }
}

#[derive(Clone, PartialEq, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum DomainConfigEntityAction {
    Read,
    Write,
}

impl ActionPermission for DomainConfigEntityAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read => PERMISSION_SET_EXPOSED_CONFIGS_VIEWER,
            Self::Write => PERMISSION_SET_EXPOSED_CONFIGS_WRITER,
        }
    }
}

impl From<DomainConfigEntityAction> for DomainConfigAction {
    fn from(action: DomainConfigEntityAction) -> Self {
        DomainConfigAction::ExposedConfig(action)
    }
}
