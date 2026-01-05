use async_graphql::*;

use domain_config::ConfigType as DomainConfigType;

use lana_app::exposed_config as app_exposed_config;

use crate::graphql::primitives::Json;

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigType {
    Bool,
    String,
    Int,
    Uint,
    Decimal,
    Complex,
}

impl From<DomainConfigType> for ConfigType {
    fn from(value: DomainConfigType) -> Self {
        match value {
            DomainConfigType::Bool => ConfigType::Bool,
            DomainConfigType::String => ConfigType::String,
            DomainConfigType::Int => ConfigType::Int,
            DomainConfigType::Uint => ConfigType::Uint,
            DomainConfigType::Decimal => ConfigType::Decimal,
            DomainConfigType::Complex => ConfigType::Complex,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct ExposedConfigItem {
    pub key: String,
    pub config_type: ConfigType,
    pub value: Json,
    pub is_set: bool,
}

impl From<app_exposed_config::ExposedConfigItem> for ExposedConfigItem {
    fn from(item: app_exposed_config::ExposedConfigItem) -> Self {
        Self {
            key: item.key,
            config_type: item.config_type.into(),
            value: item.value.into(),
            is_set: item.is_set,
        }
    }
}

#[derive(InputObject)]
pub struct ExposedConfigUpdateInput {
    pub key: String,
    pub value: Json,
}
crate::mutation_payload! { ExposedConfigUpdatePayload, exposed_config: ExposedConfigItem }
